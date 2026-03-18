---
description: Steps to set up your own Snowbridge message relayers.
---

# Run Relayers

This guide explains how to run Snowbridge relayers using Docker Compose.

## Overview

Snowbridge relayers are off-chain agents that facilitate message passing between Ethereum and Polkadot. Running a relayer helps decentralize the bridge and you can earn rewards for successfully relaying messages.

### Which Relayers Should I Run?

For new operators, we recommend starting with:

| Relayer        | Direction                          |
| -------------- | ---------------------------------- |
| `parachain-v2` | Polkadot → Ethereum, Snowbridge V2 |
| `parachain`    | Polkadot → Ethereum, Snowbridge V1 |
| `ethereum-v2`  | Ethereum → Polkadot, Snowbridge V2 |
| `ethereum`     | Ethereum → Polkadot, Snowbridge V1 |

**Note:** The `beefy` and `beacon` relayers are consensus relayers that are expensive to operate (high gas costs) and are run exclusively by the Snowfork team. Individual operators do not need to run these.

### Hardware Requirements

Minimum recommended specifications:

* **CPU:** 2 cores (dedicated, avoid burstable instances)
* **RAM:** 4 GB
* **Storage:** 20 GB SSD
* **Network:** Stable internet connection with low latency

## Prerequisites

* Docker and Docker Compose installed
* Private keys for signing transactions (Ethereum and/or Substrate)
* RPC endpoints for:
  * Ethereum execution layer (WebSocket)
  * Ethereum beacon chain (HTTP)
  * Polkadot relay chain (WebSocket)
  * BridgeHub parachain (WebSocket)
  * AssetHub parachain (WebSocket, for ethereum relay gas estimation)

## Quick Start

1.  **Download the Docker Compose file and environment template for your network:**

    ```bash
    mkdir snowbridge && cd snowbridge

    # Docker Compose file
    curl -O https://raw.githubusercontent.com/Snowfork/snowbridge/main/relayer/docker-compose.yml

    # For mainnet (Polkadot + Ethereum)
    curl -o .env https://raw.githubusercontent.com/Snowfork/snowbridge/main/relayer/.env.mainnet.example
    ```
2. **Configure your .env file with:**
   * RPC endpoints
   * Private key references (see [Private Keys](run-relayers.md#private-keys) section)
   * (Mainnet only) Chainalysis API key for OFAC compliance
3.  **Start the relayers:**

    ```bash
    docker compose up -d
    ```

To start all services including consensus relayers (Snowfork only):

```bash
docker compose --profile consensus up -d
```

## Architecture

The Docker Compose setup runs the following relayer services:

| Service                | Description                                      | Keys Required | Profile   |
| ---------------------- | ------------------------------------------------ | ------------- | --------- |
| `beacon-state-service` | Caches beacon state proofs                       | None          | default   |
| `beacon`               | Relays Ethereum beacon headers to Polkadot       | Substrate     | consensus |
| `ethereum-v2`          | Relays Ethereum messages to Polkadot (v2)        | Substrate     | default   |
| `ethereum`             | Relays Ethereum messages to Polkadot (v1)        | Substrate     | default   |
| `parachain-v2`         | Relays Polkadot messages to Ethereum (v2)        | Ethereum      | default   |
| `parachain`            | Relays Polkadot messages to Ethereum (v1)        | Ethereum      | default   |
| `primary-governance`   | Relays primary governance messages to Ethereum   | Ethereum      | default   |
| `secondary-governance` | Relays secondary governance messages to Ethereum | Ethereum      | default   |
| `reward`               | Processes relayer rewards                        | Substrate     | default   |
| `beefy`                | Relays BEEFY commitments to Ethereum             | Ethereum      | consensus |
| `beefy-on-demand`      | On-demand BEEFY relay                            | Ethereum      | consensus |

**Note:** Services in the `consensus` profile require `--profile` consensus to start.

### Service Dependencies

```
beacon-state-service (starts first, health checked)
    ├── beacon (consensus profile)
    ├── ethereum-v2
    ├── ethereum
    └── reward

parachain-v2 (independent)
parachain (independent)
primary-governance (independent)
secondary-governance (independent)
beefy (independent, consensus profile)
beefy-on-demand (independent, consensus profile)
```

## Configuration

### Environment Files

Each network has a pre-configured environment file:

| Network | File                   | Ethereum         | Polkadot |
| ------- | ---------------------- | ---------------- | -------- |
| Mainnet | `.env.mainnet.example` | Ethereum Mainnet | Polkadot |
| Paseo   | `.env.paseo.example`   | Sepolia          | Paseo    |
| Westend | `.env.westend.example` | Sepolia          | Westend  |

### Private Keys

For production deployments, use AWS Secrets Manager:

```bash
# Pattern: {environment}/{relay-name}
BEACON_RELAY_SUBSTRATE_KEY_ID=snowbridge/beacon-relay
EXECUTION_RELAY_SUBSTRATE_KEY_ID=snowbridge/asset-hub-ethereum-relay-v2
BEEFY_RELAY_ETHEREUM_KEY_ID=snowbridge/beefy-relay
BEEFY_ON_DEMAND_RELAY_ETHEREUM_KEY_ID=snowbridge/beefy-on-demand-relay
PARACHAIN_V1_RELAY_ETHEREUM_KEY_ID=snowbridge/asset-hub-parachain-relay
PARACHAIN_RELAY_ETHEREUM_KEY_ID=snowbridge/asset-hub-parachain-relay-v2
REWARD_RELAY_SUBSTRATE_KEY_ID=snowbridge/asset-hub-parachain-relay-v2-delivery-proof
EXECUTION_V1_RELAY_SUBSTRATE_KEY_ID=snowbridge/asset-hub-ethereum-relay
PRIMARY_GOVERNANCE_RELAY_ETHEREUM_KEY_ID=prod/governance-relay
SECONDARY_GOVERNANCE_RELAY_ETHEREUM_KEY_ID=prod/governance-relay
```

Create secrets in AWS Secrets Manager containing the raw private key strings. Requires AWS credentials configured in your `.env` file.

### Endpoint Configuration

All endpoints are configured via environment variables:

| Variable             | Description                              |
| -------------------- | ---------------------------------------- |
| `ETHEREUM_ENDPOINT`  | Ethereum execution layer RPC (WebSocket) |
| `BEACON_ENDPOINT`    | Ethereum beacon chain HTTP endpoint      |
| `POLKADOT_ENDPOINT`  | Polkadot relay chain RPC (WebSocket)     |
| `BRIDGEHUB_ENDPOINT` | BridgeHub parachain RPC (WebSocket)      |
| `ASSETHUB_ENDPOINT`  | AssetHub parachain RPC (WebSocket)       |
| `FLASHBOTS_ENDPOINT` | Flashbots RPC for private transactions   |

### OFAC Compliance

The execution and parachain relays support OFAC compliance checking via Chainalysis.

* **Mainnet**: Enabled by default, requires `CHAINALYSIS_API_KEY`
* **Testnets**: Disabled by default

### Fund Relayer Accounts

The Ethereum and Polkadot BridgeHub accounts should be funded with at least $10 each.

## Operations

### View logs

```bash
# All services
docker compose logs -f

# Specific service
docker compose logs -f parachain-v2
```

### Stop relayers

```bash
docker compose down
```

### Restart a specific relayer

```bash
docker compose restart ethereum-v2
```

### Check health

```bash
# Beacon state service health
curl http://localhost:8080/health

# Check container status
docker compose ps
```

### Upgrade

To upgrade to a newer relayer version:

```bash
# Pull the latest image
docker compose pull

# Restart with the new image
docker compose up -d
```

Or specify a specific version via the `IMAGE_TAG` environment variable in your `.env` file. The example `.env` files are pre-configured with the correct image tag (`snowbridge-relayers-v1`).

## Volumes

The setup creates persistent volumes for:

* `beacon-state-data` — Beacon state service cache and persistence
* `beacon-data` — Beacon relay local datastore

To reset state:

```bash
docker compose down -v
```

## Rewards

Relayers earn rewards for successfully delivering messages:

* **Polkadot → Ethereum** (`parachain-v2`): Rewards are paid in ETH on Ethereum
* **Ethereum → Polkadot** (`ethereum-v2`): Rewards are paid in DOT on AssetHub

To claim rewards, configure the `REWARD_ADDRESS` environment variable with your reward destination address.

The `reward` relayer service automatically claims accumulated rewards periodically.

## Monitoring

### CloudWatch Logging (AWS)

If running on AWS EC2, logs are automatically sent to CloudWatch when configured:

1. Set `AWS_REGION`, `AWS_ACCESS_KEY_ID`, and `AWS_SECRET_ACCESS_KEY` in your `.env` file
2. Logs will appear in CloudWatch under `snowbridge/{environment}/`

### Local Logging

```bash
# Follow all logs
docker compose logs -f

# Follow specific service
docker compose logs -f parachain-v2

# View last 100 lines
docker compose logs --tail 100 ethereum-v2
```

### Health Checks

```bash
# Beacon state service health
curl http://localhost:8080/health

# Check container status
docker compose ps
```

## Troubleshooting

### Beacon state service not healthy

Check the logs:

```bash
docker compose logs beacon-state-service
```

Common issues:

* Beacon endpoint not reachable
* Incorrect fork versions (check your .env matches the network)

### Relayer failing to submit transactions

* Check private key is correctly configured
* If using AWS Secrets Manager, verify AWS credentials in `.env`
* Check endpoint connectivity

### Gas estimation failures (ethereum relay)

* Verify AssetHub and BridgeHub endpoints are correct

### Relayer not picking up messages

* Ensure your endpoints are synced and not lagging
* Check logs for connectivity issues

## Getting Help

* Telegram: [Snowbridge Relayer Group](https://t.me/+I8Iel-Eaxcw3NjU0)
* GitHub Issues: https://github.com/Snowfork/snowbridge/issues
