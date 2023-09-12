FROM rust:1.71-bookworm AS builder

COPY . /code
WORKDIR /code
RUN cargo build --release -p buildsrs-backend

FROM debian:12

COPY --from=builder /code/target/release/buildsrs-backend /usr/local/bin
ENTRYPOINT ["/usr/local/bin/buildsrs-backend"]
