-- registered builders and their tokens
CREATE TABLE builders(
    builder_id BIGSERIAL PRIMARY KEY,
    builder_pubkey TEXT UNIQUE,
    builder_fingerprint_sha256 TEXT UNIQUE,
    builder_fingerprint_sha512 TEXT UNIQUE
);

-- targets that can be built
CREATE TABLE targets(
    target_id BIGSERIAL PRIMARY KEY,
    target_name TEXT UNIQUE
);

CREATE INDEX targets_name ON targets(target_name);

-- registry crates
CREATE TABLE registry_crates(
    crate_id BIGSERIAL PRIMARY KEY,
    crate_name TEXT UNIQUE
);

CREATE INDEX registry_crates_name ON registry_crates(crate_name);

-- registry crate versions
CREATE TABLE registry_versions(
    version_id BIGSERIAL PRIMARY KEY,
    crate_id BIGINT REFERENCES registry_crates(crate_id) ON DELETE CASCADE,
    version TEXT UNIQUE,
    checksum TEXT,
    yanked BOOLEAN,
    prerelease BOOLEAN,
    download_url TEXT
);

CREATE INDEX registry_versions_version ON registry_versions(crate_id, version);

-- view for registry versions
CREATE VIEW registry_versions_view AS
    SELECT
        registry_crates.crate_name,
        registry_versions.*
    FROM registry_crates
    JOIN registry_versions
        ON registry_crates.crate_id = registry_versions.crate_id;

-- build jobs that are running
CREATE TABLE build_jobs(
    job_id BIGSERIAL PRIMARY KEY,
    builder_id BIGINT,
    target_id BIGINT,
    version_id BIGINT,
    running BOOLEAN,
    success BOOLEAN
);

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
        AND target_id = targets.target_id
    );

-- log output from build job (json)
CREATE TABLE build_job_logs(
    build_job_log_id BIGSERIAL PRIMARY KEY,
    job_id BIGINT,
    contents TEXT
);

-- build job artifacts
CREATE TABLE build_job_artifacts(
    job_id BIGSERIAL PRIMARY KEY,
    artifact_name TEXT,
    artifact_hash TEXT,
    artifact_signature TEXT,
    download_count BIGINT
);


