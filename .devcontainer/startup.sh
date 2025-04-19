#!/bin/sh

set -xe

# nats context setting
nats context add nats --server nats:4222 --description "docker-compose NATS" --select

# minio user
mc alias set "${MINIO_ALIAS}_admin" $MINIO_URL $MINIO_ROOT_USER $MINIO_ROOT_PASSWORD
mc admin user add "${MINIO_ALIAS}_admin" $MINIO_ACCESS_KEY $MINIO_SECRET_KEY
mc admin policy attach "${MINIO_ALIAS}_admin" readwrite --user $MINIO_USER
mc alias set $MINIO_ALIAS $MINIO_URL $MINIO_ACCESS_KEY $MINIO_SECRET_KEY

# devcontainerのユーザリマッピングの範囲外なので手動で直す
sudo chown -R $USER:$USER /usr/local/cargo
