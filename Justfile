# local postgres credentials
postgres_user := "postgres"
postgres_pass := "password"
postgres_str := "host=localhost user=" + postgres_user + " password=" + postgres_pass

# list targets and help
list:
    just --list

# start postgres container
database:
    docker run -it --name buildsrs_postgres --rm -e POSTGRES_USER={{postgres_user}} -e POSTGRES_PASSWORD={{postgres_pass}} -p 127.0.0.1:5432:5432 postgres postgres -c log_statement=all

# start repl with specified postgres database
database-repl DATABASE:
    docker exec -it buildsrs_postgres psql -U postgres -d {{DATABASE}}

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

# run migrations on database
migrate:
    cargo run -p buildsrs-database --features tools --bin migrate -- --database "host=localhost user={{postgres_user}} password={{postgres_pass}}"

# launch registry sync
backend:
    RUST_LOG=debug cargo run -p buildsrs-backend -- --database "{{postgres_str}}"

# launch registry sync
registry-sync:
    RUST_LOG=debug cargo run -p buildsrs-registry-sync -- --path /tmp/registry --database "{{postgres_str}}"

# launch builder
builder:
    RUST_LOG=debug cargo run -p buildsrs-builder -- --private-key-file ~/.ssh/id_ed25519 --websocket ws://localhost:8000/api/v1/jobs

# Format source with rustfmt nightly
format:
    cargo +nightly fmt --all
