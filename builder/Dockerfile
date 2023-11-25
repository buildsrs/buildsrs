FROM rust:1.74-bookworm AS builder

COPY . /code
WORKDIR /code
RUN cargo build --release -p buildsrs-builder

FROM debian:12

COPY --from=builder /code/target/release/buildsrs-builder /usr/local/bin
ENTRYPOINT ["/usr/local/bin/buildsrs-builder"]
