-- ssh public keys
CREATE TABLE "pubkeys" (
    "id" BIGSERIAL PRIMARY KEY,
    "encoded" TEXT UNIQUE
);

-- ssh public key fingerprints
CREATE TABLE "pubkey_fingerprints" (
    "fingerprint" TEXT PRIMARY KEY,
    "pubkey" BIGINT REFERENCES pubkeys(id) ON DELETE CASCADE
);

-- registered builders
CREATE TABLE "builders" (
    "id" BIGSERIAL PRIMARY KEY,
    "uuid" UUID UNIQUE,
    "pubkey" BIGINT REFERENCES pubkeys(id) ON DELETE RESTRICT,
    "enabled" BOOLEAN DEFAULT (FALSE),
    "comment" TEXT
);

-- targets that can be built
CREATE TABLE "targets" (
    "id" BIGSERIAL PRIMARY KEY,
    "enabled" BOOLEAN DEFAULT (FALSE),
    "name" TEXT UNIQUE
);

-- targets that are enabled for every builder
CREATE TABLE "builder_targets" (
    "builder" BIGINT REFERENCES builders(id) ON DELETE CASCADE,
    "target" BIGINT REFERENCES targets(id) ON DELETE CASCADE,
    PRIMARY KEY ("builder", "target")
);

-- registry crates
CREATE TABLE "crates" (
    "id" BIGSERIAL PRIMARY KEY,
    "enabled" BOOLEAN DEFAULT (TRUE),
    "name" TEXT UNIQUE
);

-- registry crate versions
CREATE TABLE "crate_versions" (
    "id" BIGSERIAL PRIMARY KEY,
    "crate" BIGINT REFERENCES crates(id) ON DELETE CASCADE,
    "version" TEXT UNIQUE,
    "checksum" TEXT,
    "yanked" BOOLEAN,
    "prerelease" BOOLEAN,
    "download_url" TEXT
);

-- build jobs that are running
CREATE TABLE jobs(
    "id" BIGSERIAL PRIMARY KEY,
    "uuid" UUID UNIQUE,
    "builder" BIGINT REFERENCES builders(id) ON DELETE RESTRICT,
    "target" BIGINT REFERENCES targets(id) ON DELETE RESTRICT,
    "crate_version" BIGINT REFERENCES crate_versions(id) ON DELETE RESTRICT,
    "started" BIGINT,
    "ended" BIGINT,
    "success" BOOLEAN
);

-- log output from build job (json)
CREATE TABLE "job_logs" (
    "id" BIGSERIAL PRIMARY KEY,
    "job" BIGINT REFERENCES jobs(id) ON DELETE CASCADE,
    "data" TEXT
);

-- build job artifacts
CREATE TABLE "job_artifacts" (
    "id" BIGSERIAL PRIMARY KEY,
    "job" BIGINT REFERENCES jobs(id) ON DELETE CASCADE,
    "name" TEXT,
    "hash" TEXT,
    "signature" TEXT,
    "downloads" BIGINT
);

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

-- view for registry versions
CREATE VIEW crate_versions_view AS
    SELECT
        crates.name,
        crate_versions.*
    FROM crates
    JOIN crate_versions
        ON crates.id = crate_versions.crate;

-- build queue: per-target list of crate versions that we have not built yet.
CREATE VIEW build_queue AS
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
    );

