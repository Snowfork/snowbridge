#!/usr/bin/env bash
set -eux

url="http://localhost:8080/health"
echo "checking start-services and relayer state"
while true; do
  # Check if start-services.sh is still running
  pgrep -f start-services.sh > /dev/null
  if [ $? -ne 0 ]; then
    echo "start-services.sh is not running. Exiting with error."
    exit 1 # Exit with an error condition
  fi

  # Use curl to get the HTTP status code
  status_code=$(curl -o /dev/null -s -w "%{http_code}\n" "$url" || echo "000")

  # Check if the status code is 200
  if [ "$status_code" -eq 200 ]; then
    echo "Beacon relayer has started.."
    exit 0
  else
    echo "Beacon relayer has not started yet. Waiting for 10 seconds."
    sleep 10
  fi
done
