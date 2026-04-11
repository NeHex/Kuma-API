FROM rust:1 AS builder
WORKDIR /app

COPY Cargo.toml ./
COPY src ./src

RUN cargo build --release

FROM debian:bookworm-slim
WORKDIR /app

COPY --from=builder /app/target/release/kuma-api /usr/local/bin/kuma-api

EXPOSE 7788

CMD ["kuma-api"]
