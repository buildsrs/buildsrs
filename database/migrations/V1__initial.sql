-- ssh public keys
CREATE TABLE "pubkeys" (
    "id" BIGSERIAL PRIMARY KEY,
    "encoded" TEXT NOT NULL UNIQUE
);

-- ssh public key fingerprints
CREATE TABLE "pubkey_fingerprints" (
    "fingerprint" TEXT PRIMARY KEY,
    "pubkey" BIGINT NOT NULL REFERENCES pubkeys(id) ON DELETE CASCADE
);

-- builders that are registered
CREATE TABLE "builders" (
    "id" BIGSERIAL PRIMARY KEY,
    "uuid" UUID NOT NULL UNIQUE,
    "pubkey" BIGINT NOT NULL REFERENCES pubkeys(id) ON DELETE RESTRICT,
    "enabled" BOOLEAN NOT NULL DEFAULT (FALSE),
    "heartbeat" BIGINT,
    "comment" TEXT
);

-- triples that can be built for (example x86_64-unknown-linux-musl)
CREATE TABLE "triples" (
    "id" BIGSERIAL PRIMARY KEY,
    "enabled" BOOLEAN NOT NULL DEFAULT (FALSE),
    "name" TEXT NOT NULL UNIQUE
);

INSERT INTO "triples"("name", "enabled") VALUES ('generic', true);

-- triples that are enabled per builder
CREATE TABLE "builder_triples" (
    "builder" BIGINT NOT NULL REFERENCES builders(id) ON DELETE CASCADE,
    "triple" BIGINT NOT NULL REFERENCES triples(id) ON DELETE CASCADE,
    PRIMARY KEY ("builder", "triple")
);

-- crates from the registry
CREATE TABLE "crates" (
    "id" BIGSERIAL PRIMARY KEY,
    "enabled" BOOLEAN NOT NULL DEFAULT (TRUE),
    "name" TEXT NOT NULL UNIQUE
);

-- crate versions
CREATE TABLE "crate_versions" (
    "id" BIGSERIAL PRIMARY KEY,
    "crate" BIGINT NOT NULL REFERENCES crates(id) ON DELETE CASCADE,
    "version" TEXT NOT NULL UNIQUE,
    "checksum" TEXT NOT NULL,
    "yanked" BOOLEAN NOT NULL
);

-- job stages
CREATE TABLE "task_kinds" (
    "id" BIGSERIAL PRIMARY KEY,
    "name" TEXT NOT NULL UNIQUE
);

INSERT INTO task_kinds(name) VALUES ('metadata');
INSERT INTO task_kinds(name) VALUES ('tarball');
INSERT INTO task_kinds(name) VALUES ('trunk');
INSERT INTO task_kinds(name) VALUES ('coverage');

CREATE TABLE "tasks" (
    "id" BIGSERIAL PRIMARY KEY,
    "version" BIGINT NOT NULL REFERENCES crate_versions(id) ON DELETE RESTRICT,
    "kind" BIGINT NOT NULL REFERENCES task_kinds(id) ON DELETE RESTRICT,
    "triple" BIGINT NOT NULL REFERENCES triples(id) ON DELETE RESTRICT,
    UNIQUE ("version", "kind", "triple")
);

-- job stages
CREATE TABLE "job_stages" (
    "id" BIGSERIAL PRIMARY KEY,
    "name" TEXT NOT NULL UNIQUE
);

INSERT INTO job_stages(name) VALUES ('init');
INSERT INTO job_stages(name) VALUES ('fetch');
INSERT INTO job_stages(name) VALUES ('build');
INSERT INTO job_stages(name) VALUES ('upload');

-- build jobs and their current status
CREATE TABLE "jobs" (
    "id" BIGSERIAL PRIMARY KEY,
    "task" BIGINT NOT NULL REFERENCES tasks(id) ON DELETE RESTRICT,
    "uuid" UUID NOT NULL UNIQUE,
    "builder" BIGINT NOT NULL REFERENCES builders(id) ON DELETE RESTRICT,
    "started" BIGINT NOT NULL DEFAULT (0),
    "timeout" BIGINT NOT NULL DEFAULT (0),
    "stage" BIGINT NOT NULL REFERENCES job_stages(id) ON DELETE RESTRICT,
    "ended" BIGINT,
    "success" BOOLEAN
);

-- jobs log output
CREATE TABLE "job_logs" (
    "id" BIGSERIAL PRIMARY KEY,
    "job" BIGINT NOT NULL REFERENCES jobs(id) ON DELETE CASCADE,
    "stage" BIGINT NOT NULL REFERENCES job_stages(id) ON DELETE RESTRICT,
    "line" TEXT NOT NULL
);

-- build job artifacts
CREATE TABLE "job_artifacts" (
    "id" BIGSERIAL PRIMARY KEY,
    "job" BIGINT NOT NULL REFERENCES jobs(id) ON DELETE CASCADE,
    "name" TEXT NOT NULL,
    "hash" TEXT NOT NULL,
    "size" BIGINT NOT NULL,
    "signature" TEXT NOT NULL,
    "downloads" BIGINT NOT NULL DEFAULT (0)
);

-- track downloads per artifact
CREATE TABLE "job_artifact_downloads" (
    "artifact" BIGINT NOT NULL REFERENCES job_artifacts(id) ON DELETE CASCADE,
    "date" BIGINT,
    "downloads" BIGINT,
    PRIMARY KEY ("artifact", "date")
);

CREATE VIEW "tasks_view" AS
    SELECT
        crates.name AS crate,
        crate_versions.version,
        task_kinds.name AS kind,
        triples.name AS triple
    FROM tasks
    JOIN triples ON tasks.triple = triples.id
    JOIN task_kinds ON tasks.kind = task_kinds.id
    JOIN crate_versions ON tasks.version = crate_versions.id
    JOIN crates ON crate_versions.crate = crates.id;

CREATE VIEW "jobs_view" AS
    SELECT
        jobs.*,
        triples.name AS triple_name,
        builders.uuid AS builder_uuid,
        crates.name AS crate_name,
        crate_versions.version AS crate_version_version
    FROM jobs
    JOIN builders
        ON jobs.builder = builders.id
    JOIN tasks
        ON jobs.task = tasks.id
    JOIN triples
        ON tasks.triple = triples.id
    JOIN crate_versions
        ON tasks.version = crate_versions.id
    JOIN crates
        ON crate_versions.crate = crates.id;

CREATE VIEW "pubkey_fingerprints_view" AS
    SELECT
        pubkey_fingerprints.fingerprint,
        pubkeys.id,
        pubkeys.encoded
    FROM
        pubkey_fingerprints
    JOIN pubkeys
        ON pubkey_fingerprints.pubkey = pubkeys.id;

CREATE VIEW "builders_view" AS
    SELECT
        pubkeys.encoded AS pubkey,
        builders.id,
        builders.enabled,
        builders.comment,
        builders.uuid
    FROM builders
    JOIN pubkeys
        ON builders.pubkey = pubkeys.id;

CREATE VIEW "builder_triples_view" AS
    SELECT
        builders.id AS builder,
        builders.uuid AS builder_uuid,
        builders.enabled AS builder_enabled,
        builders.comment AS builder_comment,
        triples.id AS triple,
        triples.name AS triple_name
    FROM builder_triples
    JOIN triples
        ON builder_triples.triple = triples.id
    JOIN builders
        ON builder_triples.builder = builders.id;

-- view for registry versions
CREATE VIEW "crate_versions_view" AS
    SELECT
        crates.name,
        crate_versions.*
    FROM crates
    JOIN crate_versions
        ON crates.id = crate_versions.crate;

