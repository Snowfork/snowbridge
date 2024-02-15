#!/usr/bin/env bash
set -eu

url="http://localhost:8080/health"
echo "checking start-services and relayer state"
while true; do
  # Check if start-services.sh is still running
  pgrep -f start-services.sh > /dev/null
  if [ $? -ne 0 ]; then
    echo "start-services.sh is not running. Exiting with error."
    exit 1 # Exit with an error condition
  fi

  # Use curl to get the HTTP status code, trimming any trailing newlines
  status_code=$(curl -o /dev/null -s -w "%{http_code}" "$url" || echo -n 000)
  # Remove any trailing newlines or whitespace
  status_code=$(echo -n "$status_code" | tr -d '\n')

  # Check if the status code is 200
  if [ "$status_code" -eq 200 ]; then
    echo "Beacon relayer has started.."
    exit 0
  else
    echo "Beacon relayer has not started yet. Waiting for 10 seconds."
    sleep 10
  fi
done
