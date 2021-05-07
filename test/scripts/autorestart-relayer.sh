# This script can be useful for testing purposes while working on the relayer.

while true
  do
  echo "Starting relayer"
  mage dev > /tmp/snowbridge-e2e-config/relay.log 2>&1
  echo "Relayer crashed, restarting..."
  sleep 1
done
