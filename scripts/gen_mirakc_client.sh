#!/bin/sh

#    -i http://tuner:40772/api/docs \
pnpx  @openapitools/openapi-generator-cli generate \
    -i docs.json \
    -g rust \
    -o server/mirakc-client \
    --skip-validate-spec \
    --package-name mirakc-client
cargo fmt -p mirakc-client
