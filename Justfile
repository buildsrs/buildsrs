# local postgres credentials
postgres_user := "postgres"
postgres_pass := "password"
postgres_str := "host=localhost user=" + postgres_user + " password=" + postgres_pass

# local minio credentials
minio_access_key := "minio"
minio_secret_key := "secret"
minio_bucket := "bucket"

# list targets and help
list:
    just --list

# start postgres container
database:
    docker run -it --name buildsrs_postgres --rm -e POSTGRES_USER={{postgres_user}} -e POSTGRES_PASSWORD={{postgres_pass}} -p 127.0.0.1:5432:5432 postgres postgres -c log_statement=all

# start repl with specified postgres database
database-repl DATABASE:
    docker exec -it buildsrs_postgres psql -U postgres -d {{DATABASE}}

# start minio container
minio:
    docker run --name buildsrs_minio -d -p 9000:9000 -e MINIO_ACCESS_KEY={{minio_access_key}} -e MINIO_SECRET_KEY={{minio_secret_key}} minio/minio server /data
    docker run --rm --link buildsrs_minio:minio -e MINIO_BUCKET={{minio_bucket}} --entrypoint sh minio/mc -c "\
      while ! nc -z minio 9000; do echo 'Wait minio to startup...' && sleep 0.1; done; \
      sleep 5 && \
      mc config host add myminio http://minio:9000 \$MINIO_ENV_MINIO_ACCESS_KEY \$MINIO_ENV_MINIO_SECRET_KEY && \
      mc rm -r --force myminio/\$MINIO_BUCKET || true && \
      mc mb myminio/\$MINIO_BUCKET && \
      mc policy download myminio/\$MINIO_BUCKET \
    "

# run all unit tests
test filter='':
    DATABASE="{{postgres_str}}" cargo test -p buildsrs-database --all-features {{filter}}
    DATABASE="{{postgres_str}}" cargo test -p buildsrs-backend --all-features {{filter}}
    DATABASE="{{postgres_str}}" cargo test -p buildsrs-backend --all-features {{filter}}

# generate test coverage report
coverage:
    DATABASE="{{postgres_str}}" cargo llvm-cov --all-features

# launch frontend
frontend:
    cd frontend && trunk serve

# run migrations on database
migrate:
    cargo run -p buildsrs-database --features migrations --bin migrate -- host=localhost user={{postgres_user}} password={{postgres_pass}}

# launch registry sync
registry-sync:
    RUST_LOG=debug cargo run -p buildsrs-registry-sync -- --path /tmp/registry --database "{{postgres_str}}"
