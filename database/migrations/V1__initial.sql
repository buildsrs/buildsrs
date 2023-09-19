-- ssh public keys
CREATE TABLE "pubkeys" (
    "id" BIGSERIAL PRIMARY KEY,
    "encoded" TEXT UNIQUE
);

-- ssh public key fingerprints
CREATE TABLE "fingerprints" (
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
CREATE TABLE "registry_crates" (
    crate_id BIGSERIAL PRIMARY KEY,
    crate_enabled BOOLEAN DEFAULT (TRUE),
    crate_name TEXT UNIQUE
);

-- registry crate versions
CREATE TABLE "registry_versions" (
    version_id BIGSERIAL PRIMARY KEY,
    crate_id BIGINT REFERENCES registry_crates(crate_id) ON DELETE CASCADE,
    version TEXT UNIQUE,
    checksum TEXT,
    yanked BOOLEAN,
    prerelease BOOLEAN,
    download_url TEXT
);

-- build jobs that are running
CREATE TABLE build_jobs(
    job_id BIGSERIAL PRIMARY KEY,
    builder_id BIGINT,
    target_id BIGINT,
    version_id BIGINT,
    running BOOLEAN,
    success BOOLEAN
);

-- log output from build job (json)
CREATE TABLE "build_job_logs" (
    build_job_log_id BIGSERIAL PRIMARY KEY,
    job_id BIGINT,
    contents TEXT
);

-- build job artifacts
CREATE TABLE "build_job_artifacts" (
    job_id BIGSERIAL PRIMARY KEY,
    artifact_name TEXT,
    artifact_hash TEXT,
    artifact_signature TEXT,
    download_count BIGINT
);

CREATE VIEW "fingerprints_view" AS
    SELECT
        fingerprints.fingerprint,
        pubkeys.id,
        pubkeys.encoded
    FROM
        fingerprints
    JOIN pubkeys
        ON fingerprints.pubkey = pubkeys.id;

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
CREATE VIEW registry_versions_view AS
    SELECT
        registry_crates.crate_name,
        registry_versions.*
    FROM registry_crates
    JOIN registry_versions
        ON registry_crates.crate_id = registry_versions.crate_id;

-- build queue: per-target list of crate versions that we have not built yet.
CREATE VIEW build_queue AS
    SELECT
        targets.*,
        registry_versions.*
    FROM targets
    CROSS JOIN registry_versions
    WHERE registry_versions.yanked = FALSE
    AND NOT EXISTS (
        SELECT job_id
        FROM build_jobs
        WHERE version_id = registry_versions.version_id
        AND target_id = targets.id
    );

