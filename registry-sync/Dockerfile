FROM rust:1.71-bookworm AS builder

# install dependencies
RUN apt update && \
    apt install -y cmake && \
    rm -rf /var/lib/apt/lists/*

# build
COPY . /code
WORKDIR /code
RUN cargo build --release -p buildsrs-registry-sync

FROM debian:12

COPY --from=builder /code/target/release/buildsrs-registry-sync /usr/local/bin
ENTRYPOINT ["/usr/local/bin/buildsrs-registry-sync"]

