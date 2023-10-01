# local postgres credentials
postgres_user := "postgres"
postgres_pass := "password"
postgres_str := "host=localhost user=" + postgres_user + " password=" + postgres_pass

# list targets and help
list:
    just --list

# start postgres database
database:
    docker run -it --name buildsrs_postgres --rm -e POSTGRES_USER={{postgres_user}} -e POSTGRES_PASSWORD={{postgres_pass}} -p 127.0.0.1:5432:5432 postgres postgres -c log_statement=all

# start repl with specified postgres database
database-repl DATABASE:
    docker exec -it buildsrs_postgres psql -U postgres -d {{DATABASE}}

# save database dump
database-dump NAME='latest' DATABASE='postgres':
    docker exec -it buildsrs_postgres pg_dump -U postgres -d {{DATABASE}} --inserts | xz > database/dumps/{{NAME}}.sql.xz

# test database
database-test:
    DATABASE="{{postgres_str}}" cargo test -p buildsrs-database --all-features

# run database cli
database-cli *COMMAND:
    cargo run -p buildsrs-database --features tools -- --database "{{postgres_str}}" {{COMMAND}}

# run all unit tests
test filter='':
    DATABASE="{{postgres_str}}" cargo test -p buildsrs-database -p buildsrs-backend -p buildsrs-builder -p buildsrs-common -p buildsrs-protocol --all-features {{filter}}
    DATABASE="{{postgres_str}}" cargo test -p buildsrs-registry-sync --all-features {{filter}}

# generate test coverage report
coverage:
    DATABASE="{{postgres_str}}" cargo llvm-cov --all-features

# launch frontend
frontend:
    cd frontend && trunk serve

# launch registry sync
backend:
    RUST_LOG=debug cargo run -p buildsrs-backend -- --database "{{postgres_str}}"

# launch registry sync
registry-sync:
    RUST_LOG=debug cargo run -p buildsrs-registry-sync -- --path /tmp/registry --database "{{postgres_str}}"

# launch builder
builder:
    RUST_LOG=debug cargo run -p buildsrs-builder -- --private-key-file ~/.ssh/id_ed25519 connect --websocket ws://localhost:8000/api/v1/jobs

# Format source with rustfmt nightly
format:
    cargo +nightly fmt --all
