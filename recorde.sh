#!/bin/sh

set -xe

PROGRAM_ID=$1

if [ "${PROGRAM_ID}" = "" ]; then
  echo "Usage: $0 <program_id>"
  exit 1
fi

curl -s -X POST -H "Content-Type: application/json" -d "{\"programId\": \"${PROGRAM_ID}\", \"options\":{\"contentPath\":\"${PROGRAM_ID}.ts\"}}" http://recorder:40772/api/recordings
