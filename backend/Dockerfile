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

# install trunk for building webassembly frontends
ARG TRUNK_VERSION=0.18.3
RUN curl -sSL https://github.com/thedodd/trunk/releases/download/v${TRUNK_VERSION}/trunk-x86_64-unknown-linux-gnu.tar.gz | tar -xzf- -C /usr/local/bin

COPY . /code

# build frontend
WORKDIR /code/frontend
RUN trunk build --release

# build backend
WORKDIR /code
RUN cargo build --release -p buildsrs-backend

FROM debian:12

COPY --from=builder /code/target/release/buildsrs-backend /usr/local/bin
COPY --from=builder /code/frontend/dist /usr/share/buildsrs-frontend

ENTRYPOINT ["/usr/local/bin/buildsrs-backend"]
