FROM debian:12 AS builder

# install dependencies
RUN apt update && \
    apt install -y curl cmake pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/*

# install rustup
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain none
ENV PATH="/root/.cargo/bin:$PATH"

# update toolchain
WORKDIR /code
COPY rust-toolchain.toml .
RUN rustup show

# build registry-sync
COPY . /code
RUN cargo build --release -p buildsrs-registry-sync

# release in new container
FROM debian:12
COPY --from=builder /code/target/release/buildsrs-registry-sync /usr/local/bin
ENTRYPOINT ["/usr/local/bin/buildsrs-registry-sync"]

