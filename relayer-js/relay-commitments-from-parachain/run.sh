#!/bin/bash

trap 'kill $(jobs -p)' SIGINT SIGTERM EXIT

while :
do
  yarn relay-commitments-from-parachain 2>&1 &
  RELAY_PID=$!
  while kill -0 $RELAY_PID ; do
      sleep 5
  done
done
