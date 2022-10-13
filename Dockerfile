# FROM rust:1.63.0 AS builder
FROM lukemathwalker/cargo-chef:latest-rust-1.63.0 as chef
WORKDIR /app
RUN apt update && apt install lld clang -y
FROM chef as planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef as builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef book --release --recipe-path recipe.json
COPY . .

ENV SQLX_OFFLINE true
RUN cargo build --release --bin test2pro

# FROM rust:1.63.0-slim AS runtime
FROM debian:bullseye-20190708-slim AS runtime
WORKDIR /app
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl ca-certificates \
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/test2pro test2pro
COPY configuration configuration
ENV APP_ENVIRONMENT production
ENTRYPOINT [ "./test2pro" ]
# ENTRYPOINT ["./target/release/test2pro"]