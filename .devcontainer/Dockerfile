FROM quay.io/minio/minio as minio

FROM mcr.microsoft.com/devcontainers/rust:1-1-bookworm as devcontainer

RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
    && apt-get -y install --no-install-recommends \
    clang \
    ffmpeg \
    mold \
    openjdk-17-jre \
    protobuf-compiler \
    && apt-get clean && rm -rf /var/lib/apt/lists/*

RUN curl -sf https://binaries.nats.dev/nats-io/natscli/nats@latest | sh && mv nats /usr/local/bin/nats

COPY  --from=minio /usr/bin/mc /usr/local/bin/mc

USER vscode
ENV CARGO_HOME=/home/vscode/.cargo
RUN cargo install typeshare-cli
RUN cargo install zellij
