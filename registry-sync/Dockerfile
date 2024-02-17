FROM registry.gitlab.com/buildsrs/buildsrs/rust AS builder

# build registry-sync
COPY . /code
RUN cargo build --release -p buildsrs-registry-sync

# release in new container
FROM debian:12
COPY --from=builder /code/target/release/buildsrs-registry-sync /usr/local/bin
ENTRYPOINT ["/usr/local/bin/buildsrs-registry-sync"]

