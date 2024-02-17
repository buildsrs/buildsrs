FROM debian:12 AS builder

# install dependencies
RUN apt update && \
    DEBIAN_FRONTEND=noninteractive \
    apt install -y --no-install-recommends curl cmake pkg-config libssl-dev clang ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# install rustup
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain none
ENV PATH="/root/.cargo/bin:$PATH"

# update toolchain
WORKDIR /code
COPY rust-toolchain.toml .
RUN rustup show

# install tooling
ARG TRUNK_VERSION=0.18.3
ARG SCCACHE_VERSION=0.7.4
ARG CARGO_LLVM_COV_VERSION=0.5.39
ARG CARGO_HACK_VERSION=0.6.15
ARG CARGO_DENY_VERSION=0.14.3
ARG MOLD_VERSION=2.4.0

RUN curl -sSL https://github.com/thedodd/trunk/releases/download/v${TRUNK_VERSION}/trunk-x86_64-unknown-linux-gnu.tar.gz | tar -xzf- -C /usr/local/bin && \
    curl -sSL "https://github.com/mozilla/sccache/releases/download/v$SCCACHE_VERSION/sccache-v$SCCACHE_VERSION-x86_64-unknown-linux-musl.tar.gz" | tar -xzf- -C /usr/local/bin --strip-components=1 && \
    curl -sSL "https://github.com/taiki-e/cargo-llvm-cov/releases/download/v${CARGO_LLVM_COV_VERSION}/cargo-llvm-cov-aarch64-unknown-linux-musl.tar.gz" | tar zxv -C /usr/local/bin && \
    curl -sSL "https://github.com/taiki-e/cargo-hack/releases/download/v0.6.13/cargo-hack-x86_64-unknown-linux-musl.tar.gz" | tar zxv -C /usr/local/bin && \
    curl -sSL "https://github.com/EmbarkStudios/cargo-deny/releases/download/$CARGO_DENY_VERSION/cargo-deny-$CARGO_DENY_VERSION-x86_64-unknown-linux-musl.tar.gz" | tar zxv -C /usr/local/bin --strip-components=1 && \
    curl -sSL "https://github.com/rui314/mold/releases/download/v$MOLD_VERSION/mold-$MOLD_VERSION-x86_64-linux.tar.gz" | tar --strip-components=1 -zxv -C /usr/local

# set environment
ENV RUSTC_WRAPPER=/usr/local/bin/sccache
ENV CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=clang
ENV CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS="-C link-arg=--ld-path=/usr/local/bin/mold"
