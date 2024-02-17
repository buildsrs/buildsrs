FROM registry.gitlab.com/buildsrs/buildsrs/rust AS builder

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
