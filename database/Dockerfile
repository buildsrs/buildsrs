FROM registry.gitlab.com/buildsrs/buildsrs/rust AS builder

# build
COPY . /code
WORKDIR /code
RUN cargo build --release -p buildsrs-database --features cli

FROM debian:12

COPY --from=builder /code/target/release/buildsrs-database /usr/local/bin
ENTRYPOINT ["/usr/local/bin/buildsrs-database"]


