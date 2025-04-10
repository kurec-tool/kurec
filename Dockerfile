# TODO fix version
FROM rust:latest AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    # build-essential \
    # clang \
    curl \
    # mold \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY ./server ./server

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/src/target \
    cargo build --release && \
    mv target/release/kurec /usr/local/bin/kurec

# TODO: multistage化
# FROM debian:bookworm-slim
# RUN apt-get update \
#     && apt-get install -y --no-install-recommends \
#     ca-certificates \
#     libssl3 \
#     curl \
#     && rm -rf /var/lib/apt/lists/*
# COPY --from=builder /app/target/release/kurec /usr/local/bin/kurec
