FROM registry.gitlab.com/buildsrs/buildsrs/rust AS builder

COPY . /code
RUN cargo build --release -p buildsrs-builder

FROM debian:12

COPY --from=builder /code/target/release/buildsrs-builder /usr/local/bin
ENTRYPOINT ["/usr/local/bin/buildsrs-builder"]
