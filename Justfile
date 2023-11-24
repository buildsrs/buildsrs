# local postgres credentials
postgres_user := "postgres"
postgres_pass := "password"
postgres_str := "host=localhost user=" + postgres_user + " password=" + postgres_pass
postgres_env := "DATABASE=\"" + postgres_str + "\""

# local storage credentials
storage_endpoint := "http://localhost:9000"
storage_user := "buildsrs"
storage_pass := "password"
storage_port := "9000"
storage_env := "MINIO_ENDPOINT=" + storage_endpoint + " MINIO_USER=" + storage_user + " MINIO_PASS=" + storage_pass

# environment for services
services_env := postgres_env + " " + storage_env

# list targets and help
list:
    just --list

# launch services, pass 'down' as command to stop.
services *COMMAND='up':
    docker compose {{COMMAND}}

# start repl with specified postgres database
database-repl DATABASE:
    docker compose exec database psql -U postgres -d {{DATABASE}}

# save database dump
database-dump NAME='latest' DATABASE='postgres':
    docker compose exec database pg_dump -U postgres -d {{DATABASE}} --inserts | xz > database/dumps/{{NAME}}.sql.xz

# run database cli
database-cli *COMMAND:
    cargo run -p buildsrs-database --features tools -- --database "{{postgres_str}}" {{COMMAND}}

# run unit tests for CRATE only
test-crate CRATE *ARGS:
    DATABASE="{{postgres_str}}" cargo test -p buildsrs-{{CRATE}} --all-features {{ARGS}}

# run all unit tests
test FILTER='':
    {{services_env}} cargo test --all-features {{FILTER}}

# generate test coverage report
coverage:
    {{services_env}} cargo llvm-cov --all-features

# launch frontend
frontend:
    cd frontend && trunk serve

# launch backend
backend:
    RUST_LOG=debug cargo run -p buildsrs-backend -- --database "{{postgres_str}}" --storage filesystem --storage-filesystem-path $(mktemp)

# launch registry sync
registry-sync:
    RUST_LOG=debug cargo run -p buildsrs-registry-sync -- --path /tmp/registry --database "{{postgres_str}}"

# launch builder
builder:
    RUST_LOG=debug cargo run -p buildsrs-builder -- --private-key-file ~/.ssh/id_ed25519 connect --websocket ws://localhost:8000/api/v1/jobs

# Format source with rustfmt nightly
format:
    cargo +nightly fmt --all

# run formatting and style checks
check:
    cargo +nightly fmt --check --all
    cargo clippy --workspace --all-features -- -D warnings
