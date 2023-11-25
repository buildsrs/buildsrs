FROM rust:1.74-bookworm AS builder

# build
COPY . /code
WORKDIR /code
RUN cargo build --release -p buildsrs-database --features tools --bin migrate

FROM debian:12

COPY --from=builder /code/target/release/migrate /usr/local/bin
ENTRYPOINT ["/usr/local/bin/migrate"]


