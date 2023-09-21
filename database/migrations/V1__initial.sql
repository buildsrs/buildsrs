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
    "connected" BOOLEAN NOT NULL DEFAULT (FALSE),
    "last" BIGINT,
    "comment" TEXT
);

-- targets that can be built for (example x86_64-unknown-linux-musl)
CREATE TABLE "targets" (
    "id" BIGSERIAL PRIMARY KEY,
    "enabled" BOOLEAN NOT NULL DEFAULT (FALSE),
    "name" TEXT NOT NULL UNIQUE
);

-- targets that are enabled per builder
CREATE TABLE "builder_targets" (
    "builder" BIGINT NOT NULL REFERENCES builders(id) ON DELETE CASCADE,
    "target" BIGINT NOT NULL REFERENCES targets(id) ON DELETE CASCADE,
    PRIMARY KEY ("builder", "target")
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
CREATE TABLE job_stages(
    "id" BIGSERIAL PRIMARY KEY,
    "name" TEXT NOT NULL UNIQUE
);

INSERT INTO job_stages(name) VALUES ('fetch');
INSERT INTO job_stages(name) VALUES ('build');
INSERT INTO job_stages(name) VALUES ('upload');

-- build jobs and their current status
CREATE TABLE jobs(
    "id" BIGSERIAL PRIMARY KEY,
    "uuid" UUID NOT NULL UNIQUE,
    "builder" BIGINT NOT NULL REFERENCES builders(id) ON DELETE RESTRICT,
    "target" BIGINT NOT NULL REFERENCES targets(id) ON DELETE RESTRICT,
    "crate_version" BIGINT NOT NULL REFERENCES crate_versions(id) ON DELETE RESTRICT,
    "started" BIGINT NOT NULL,
    "timeout" BIGINT NOT NULL,
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

CREATE VIEW "jobs_view" AS
    SELECT
        jobs.*,
        targets.name AS target_name,
        crate_versions.version AS crate_version_version
    FROM jobs
    JOIN builders
        ON jobs.builder = builders.id
    JOIN targets
        ON jobs.target = targets.id
    JOIN crate_versions
        ON jobs.crate_version = crate_versions.id;

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

CREATE VIEW "builder_targets_view" AS
    SELECT
        builders.uuid AS builder_uuid,
        builders.enabled AS builder_enabled,
        builders.comment AS builder_comment,
        targets.name AS target_name
    FROM builder_targets
    JOIN targets
        ON builder_targets.target = targets.id
    JOIN builders
        ON builder_targets.builder = builders.id;

-- view for registry versions
CREATE VIEW "crate_versions_view" AS
    SELECT
        crates.name,
        crate_versions.*
    FROM crates
    JOIN crate_versions
        ON crates.id = crate_versions.crate;

-- build queue: per-target list of crate versions that we have not built yet.
CREATE VIEW "build_queue" AS
    SELECT
        targets.id AS target,
        targets.name AS target_name,
        crate_versions.id AS version_id,
        crate_versions.*
    FROM targets
    CROSS JOIN crate_versions
    WHERE crate_versions.yanked = FALSE
    AND NOT EXISTS (
        SELECT id
        FROM jobs
        WHERE crate_version = crate_versions.id
        AND target = targets.id
        AND success IS NULL OR success = TRUE
    );

