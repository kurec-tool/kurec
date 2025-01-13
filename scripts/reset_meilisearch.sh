#!/bin/sh

set -xe

indexes=$(curl -s -X GET 'http://meilisearch:7700/indexes' | jq -r '.results[].uid')
for uid in $indexes; do
    curl -s -X DELETE "http://meilisearch:7700/indexes/$uid"
    echo
done

