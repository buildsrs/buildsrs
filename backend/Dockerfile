FROM rust:1.74-bookworm AS builder

# install support for building webassembly frontends
ARG TRUNK_VERSION="0.18.3"
RUN rustup target add wasm32-unknown-unknown
RUN wget -qO- https://github.com/thedodd/trunk/releases/download/v${TRUNK_VERSION}/trunk-x86_64-unknown-linux-gnu.tar.gz | tar -xzf- -C /usr/local/bin

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
