ARG BUILDER_IMAGE=docker.m.daocloud.io/library/rust:1
ARG RUNTIME_IMAGE=docker.m.daocloud.io/library/debian:bookworm-slim

FROM ${BUILDER_IMAGE} AS builder
WORKDIR /app

COPY .cargo ./.cargo
COPY Cargo.toml Cargo.lock ./

# Pre-build dependencies to leverage Docker layer cache.
RUN mkdir -p src \
    && printf 'fn main() {}\n' > src/main.rs \
    && cargo build --release --locked \
    && rm -rf src

COPY src ./src
# Ensure real sources are newer than warm-up artifacts, then rebuild real binary.
RUN find src -type f -exec touch {} + \
    && rm -f target/release/kuma-api \
    && cargo build --release --locked

FROM ${RUNTIME_IMAGE}
WORKDIR /app

COPY --from=builder /app/target/release/kuma-api /usr/local/bin/kuma-api

EXPOSE 7788

CMD ["kuma-api"]
