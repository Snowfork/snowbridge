# Importing Beacon States

This guide explains how to manually import beacon states into the beacon-state-service store. This is useful when:

- A specific historical beacon state is required
- The primary beacon node has pruned old states
- You need to pre-populate the store from an archive node

## Prerequisites

1. Two beacon state SSZ files:
   - **Attested state**: The beacon state at the attested slot
   - **Finalized state**: The beacon state at the finalized slot (referenced by the attested state's finalized checkpoint)

2. The attested and finalized states must form a valid pair (the attested state's finalized checkpoint must reference the finalized state).

## Downloading Beacon States

You can download beacon states from any beacon node using curl:

```bash
# Download attested state (replace SLOT and BEACON_URL)
curl -H "Accept: application/octet-stream" \
  "https://BEACON_URL/eth/v2/debug/beacon/states/ATTESTED_SLOT" \
  -o attested_state.ssz

# Download finalized state
curl -H "Accept: application/octet-stream" \
  "https://BEACON_URL/eth/v2/debug/beacon/states/FINALIZED_SLOT" \
  -o finalized_state.ssz
```

Example with real values:
```bash
# Download from an archive node
curl -H "Accept: application/octet-stream" \
  "https://archive-beacon-node.example.com/eth/v2/debug/beacon/states/13572000" \
  -o attested_state.ssz

curl -H "Accept: application/octet-stream" \
  "https://archive-beacon-node.example.com/eth/v2/debug/beacon/states/13571968" \
  -o finalized_state.ssz
```

## Method 1: Import While Service is Stopped (Recommended)

This is the safest method as it avoids any SQLite locking issues.

### Step 1: Stop the beacon-state-service

```bash
docker compose stop beacon-state-service
```

### Step 2: Copy SSZ files to the mounted volume

```bash
# Find the volume mount path from docker-compose.yml
# Default is ./data or a named volume

# Copy files to accessible location
cp attested_state.ssz ./data/
cp finalized_state.ssz ./data/
```

### Step 3: Run the import command

```bash
# Run import using the relay binary
./build/snowbridge-relay import-beacon-state \
  --config ./config/beacon-relay.json \
  --attested-state-file ./data/attested_state.ssz \
  --finalized-state-file ./data/finalized_state.ssz
```

**Important:** The `--config` file must have `source.beacon.datastore.location` set to the same path the beacon-state-service uses.

### Step 4: Restart the beacon-state-service

```bash
docker compose start beacon-state-service
```

### Step 5: Verify the import

```bash
# Check the service logs
docker compose logs beacon-state-service

# Or query the health endpoint
curl http://localhost:8080/health
```

## Method 2: Import Inside Running Container

Use this method if you cannot stop the service, but be aware of potential SQLite locking issues.

### Step 1: Copy SSZ files into the container

```bash
# Copy files to the container's data volume
docker compose cp attested_state.ssz beacon-state-service:/data/
docker compose cp finalized_state.ssz beacon-state-service:/data/
```

### Step 2: Run import inside the container

```bash
docker compose exec beacon-state-service /usr/local/bin/snowbridge-relay import-beacon-state \
  --config /config/beacon-state-service.json \
  --attested-state-file /data/attested_state.ssz \
  --finalized-state-file /data/finalized_state.ssz
```

### Step 3: Clean up

```bash
docker compose exec beacon-state-service rm /data/attested_state.ssz /data/finalized_state.ssz
```

## Finding Valid Slot Pairs

To find a valid attested/finalized slot pair:

1. **Finalized slot**: Must be at an epoch boundary (slot divisible by 32)
2. **Attested slot**: Typically 2 epochs (64 slots) after the finalized slot

Example:
- Finalized slot: 13571968 (13571968 % 32 = 0 ✓)
- Attested slot: 13572032 (13571968 + 64)

You can also query the beacon node for the current finalized checkpoint:

```bash
curl "https://BEACON_URL/eth/v1/beacon/states/head/finality_checkpoints" | jq
```

## Troubleshooting

### "state pair validation failed"

The attested state's finalized checkpoint doesn't match the provided finalized state. Ensure:
- The finalized state is at the correct slot
- Both states are from the same beacon chain

### "SQLITE_BUSY" or database locked errors

Another process is writing to the database. Either:
- Stop the beacon-state-service first (Method 1)
- Wait and retry

### "unmarshal beacon state" errors

The SSZ file might be:
- Corrupted during download
- From a different fork version than expected
- Not in SSZ format (check you used `Accept: application/octet-stream`)

### Config path mismatch

Ensure the config file's datastore location matches where the beacon-state-service stores data:
- Beacon relay config: `source.beacon.datastore.location`
- Beacon-state-service config: `beacon.datastore.location`

Both should point to the same directory (e.g., `/data/beacon-state`).
