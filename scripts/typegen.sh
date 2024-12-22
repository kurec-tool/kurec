#!/bin/sh

RUST_LOG=info typeshare ./ --lang=typescript --output-folder=web/src/types
