#!/bin/sh

set -xe

nats stream ls -j | jq -r .[] | xargs -n 1 nats stream rm -f
