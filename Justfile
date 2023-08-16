# local postgres credentials
postgres_user := "postgres"
postgres_pass := "password"

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
    DATABASE="host=localhost user={{postgres_user}} password={{postgres_pass}}" cargo test --all-features {{filter}}

# generate test coverage report
coverage:
    DATABASE="host=localhost user={{postgres_user}} password={{postgres_pass}}" cargo llvm-cov --all-features

# launch frontend
frontend:
    cd frontend && trunk serve

# run migrations on database
migrate:
    cargo run -p buildsrs-database --features migrations --bin migrate -- host=localhost user={{postgres_user}} password={{postgres_pass}}

# launch registry sync
registry-sync:
    RUST_LOG=debug cargo run -p buildsrs-registry-sync -- --path /tmp/registry --database "host=localhost user={{postgres_user}} password={{postgres_pass}}"