#!/usr/bin/env bash
set -eu

url="http://localhost:8080/health"

while true; do
  status_code=$(curl -o /dev/null -s -w "%{http_code}\n" "$url")

  if [ "$status_code" -eq 200 ]; then
    echo "Beacon relayer has started.."
    exit 0
  else
    echo "Beacon relayer has not started yet. Waiting for 10 seconds."
    sleep 10
  fi
done
