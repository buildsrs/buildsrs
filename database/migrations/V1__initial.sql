-- registered builders and their tokens
CREATE TABLE builders(
    builder_id BIGSERIAL PRIMARY KEY,
    builder_token_hash TEXT
);

-- registry crates
CREATE TABLE registry_crates(
    crate_id BIGSERIAL PRIMARY KEY,
    name TEXT
);

-- registry crate versions
CREATE TABLE registry_crate_versions(
    version_id BIGSERIAL PRIMARY KEY,
    crate_id INTEGER,
    version TEXT,
    checksum TEXT,
    yanked BOOLEAN,
    prerelease BOOLEAN
);
