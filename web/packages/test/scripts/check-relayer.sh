#!/usr/bin/env bash
set -eu

# Set a timeout of 40 minutes (2400 seconds)
timeout=2400
start_time=$(date +%s)

url="http://localhost:8080/beacon/health"
echo "checking start-services and relayer state"

while true; do
  # Check if the timeout has been reached
  current_time=$(date +%s)
  elapsed_time=$((current_time - start_time))

  if [ $elapsed_time -ge $timeout ]; then
    echo "Timeout reached. Exiting with error."
    exit 1 # Exit with an error condition
  fi

  # Check if start-services.sh is still running
  pgrep -f start-services.sh > /dev/null
  if [ $? -ne 0 ]; then
    echo "start-services.sh is not running. Exiting with error."
    exit 1 # Exit with an error condition
  fi

  # Check if the beacon relayer is up
  if pgrep -f " run beacon" > /dev/null; then
      echo "Beacon relayer has started.."
      exit 0
  else
      echo "Beacon relayer has not started yet. Waiting for 10 seconds."
      sleep 10
  fi
done
