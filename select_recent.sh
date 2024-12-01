#!/bin/sh

set -xe

curl -s http://tuner:40772/api/programs | jq --arg now "$(date +%s%3N)" '
  map(select(.startAt > ($now | tonumber))) |
  min_by(.startAt + .duration) |
  {id, name, startAt: (.startAt / 1000 + 9 * 3600 | strftime("%Y-%m-%dT%H:%M:%S+09:00")), duration, endAt: ((.startAt + .duration) / 1000 + 9 * 3600 | strftime("%Y-%m-%dT%H:%M:%S+09:00"))}
'
