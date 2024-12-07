FROM quay.io/minio/minio as minio

FROM mcr.microsoft.com/devcontainers/rust:1-1-bookworm as devcontainer

RUN curl -sf https://binaries.nats.dev/nats-io/natscli/nats@latest | sh && mv nats /usr/local/bin/nats

COPY  --from=minio /usr/bin/mc /usr/local/bin/mc
