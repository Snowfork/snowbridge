# Running Snowbridge Relayers

This guide explains how to run Snowbridge relayers using Docker Compose.

## Overview

Snowbridge relayers are off-chain agents that facilitate message passing between Ethereum and Polkadot. Running a relayer helps decentralize the bridge and you can earn rewards for successfully relaying messages.

### Which Relayers Should I Run?

For new operators, we recommend starting with:

| Relayer | Direction | Reward Potential | Complexity |
|---------|-----------|------------------|------------|
| `parachain-v3` | Polkadot → Ethereum | High | Medium |
| `ethereum-v3` | Ethereum → Polkadot | Medium | Low |

**Note:** Ethereum → Polkadot light-client updates are handled inside `ethereum-v3` (no standalone `beacon` service).

### Hardware Requirements

Minimum recommended specifications:
- **CPU:** 2 cores
- **RAM:** 4 GB
- **Storage:** 20 GB SSD
- **Network:** Stable internet connection with low latency

### Cost Considerations

- **Ethereum relayers** (`parachain-v3`): Require ETH for gas fees when submitting proofs to Ethereum
- **Polkadot relayers** (`ethereum-v3`): Require DOT/KSM for transaction fees (very low cost)
- **RPC endpoints**: You'll need access to archive nodes (can use public endpoints or run your own)

## Prerequisites

- Docker and Docker Compose installed
- Private keys for signing transactions (Ethereum and/or Substrate)
- RPC endpoints for:
  - Ethereum execution layer (WebSocket)
  - Ethereum beacon chain (HTTP)
  - Polkadot relay chain (WebSocket)
  - BridgeHub parachain (WebSocket)
  - AssetHub parachain (WebSocket, for ethereum relay gas estimation)

## Quick Start

### Option A: Run All Relayers (Full Setup)

1. **Copy the environment file for your network:**
   ```bash
   # For mainnet (Polkadot + Ethereum)
   cp .env.mainnet.example .env

   # For Paseo testnet (Paseo + Sepolia)
   cp .env.paseo.example .env

   # For Westend testnet (Westend + Sepolia)
   cp .env.westend.example .env
   ```

2. **Configure your .env file with:**
   - RPC endpoints
   - Private key references (see Private Keys section)
   - (Mainnet only) Chainalysis API key for OFAC compliance

3. **Start the relayers:**
   ```bash
   docker compose up -d
   ```

### Option B: Run a Single Relayer (Recommended for Beginners)

Example: Running only the `parachain-v3` relayer on mainnet:

1. **Create your .env file:**
   ```bash
   cp .env.mainnet.example .env
   ```

2. **Edit .env with your RPC endpoints and Ethereum private key**

3. **Start only the parachain-v3 relayer:**
   ```bash
   docker compose up -d parachain-v3
   ```

This is the simplest way to start earning rewards by relaying Polkadot → Ethereum messages.

## Architecture

The Docker Compose setup runs the following relayer services:

| Service | Description | Keys Required |
|---------|-------------|---------------|
| `beacon-state-service` | Caches beacon state proofs | None |
| `ethereum-v3` | Relays Ethereum messages + LC updates to Polkadot | Substrate |
| `parachain-v3` | Relays Polkadot messages to Ethereum | Ethereum |
| `reward` | Processes relayer rewards | Substrate |

### Service Dependencies

```
beacon-state-service (starts first, health checked)
    ├── ethereum-v3  (LC catch-up + V1/V2 delivery)
    └── reward

parachain-v3 (independent)
```

## Configuration

### Environment Files

Each network has a pre-configured environment file with the correct values:

| Network | File | Ethereum | Polkadot |
|---------|------|----------|----------|
| Mainnet | `.env.mainnet.example` | Ethereum Mainnet | Polkadot |
| Paseo | `.env.paseo.example` | Sepolia | Paseo |
| Westend | `.env.westend.example` | Sepolia | Westend |

The environment files include:
- Fork versions (network-specific)
- Contract addresses (network-specific)
- Schedule parameters (mainnet vs testnet defaults)
- OFAC settings (enabled on mainnet, disabled on testnets)

### Private Keys

There are three options for providing private keys:

#### Option 1: Environment Variable (Simplest)

Set the private key directly in your `.env` file:

```bash
# For Substrate relayers (ethereum-v3, reward)
# Use the secret seed phrase or hex-encoded private key
ETHEREUM_V3_RELAY_SUBSTRATE_KEY="//Alice"  # Dev account
ETHEREUM_V3_RELAY_SUBSTRATE_KEY="0x..."    # Hex private key

# For Ethereum relayers (parachain-v3)
PARACHAIN_RELAY_ETHEREUM_KEY="0x..."  # Hex private key (without 0x prefix)
```


Then update docker-compose.yml to use `--substrate.private-key` or `--ethereum.private-key` instead of the `-id` variants.

#### Option 2: Private Key File

Store the key in a file and mount it:

```bash
echo "0x..." > /path/to/keyfile
chmod 600 /path/to/keyfile
```

Use `--substrate.private-key-file /path/to/keyfile` in the command.

#### Option 3: AWS Secrets Manager (Production)

For production deployments, use AWS Secrets Manager:

```bash
# Pattern: {network}/{relay-name}
ETHEREUM_V3_RELAY_SUBSTRATE_KEY_ID=mainnet/ethereum-relay-v2
```

Create secrets in AWS Secrets Manager containing the raw private key strings. Requires AWS credentials configured.

### Endpoint Configuration

All endpoints are configured via environment variables:

| Variable | Description |
|----------|-------------|
| `ETHEREUM_ENDPOINT` | Ethereum execution layer RPC (WebSocket) |
| `BEACON_ENDPOINT` | Ethereum beacon chain HTTP endpoint |
| `POLKADOT_ENDPOINT` | Polkadot relay chain RPC (WebSocket) |
| `BRIDGEHUB_ENDPOINT` | BridgeHub parachain RPC (WebSocket) |
| `ASSETHUB_ENDPOINT` | AssetHub parachain RPC (WebSocket) |
| `FLASHBOTS_ENDPOINT` | Flashbots RPC for private transactions |

### OFAC Compliance

The execution and parachain relays support OFAC compliance checking via Chainalysis.

- **Mainnet**: Enabled by default, requires `CHAINALYSIS_API_KEY`
- **Testnets**: Disabled by default

## Operations

### View logs

```bash
# All services
docker compose logs -f

# Specific service
docker compose logs -f ethereum-v3
```

### Stop relayers

```bash
docker compose down
```

### Restart a specific relayer

```bash
docker compose restart ethereum-v3
```

### Check health

```bash
# Beacon state service health
curl http://localhost:8080/health
```

## Volumes

The setup creates persistent volumes for:
- `beacon-state-data` - Beacon state service cache and persistence
- `ethereum-v3-data` - ethereum-v3 local datastore

To reset state:
```bash
docker compose down -v
```

## Contract Addresses

Pre-configured in the environment files:

### Mainnet (Polkadot + Ethereum)
- Gateway: `0x27ca963c279c93801941e1eb8799c23f407d68e7`
- BeefyClient: `0x1817874feab3ce053d0f40abc23870db35c2affc`

### Paseo (Paseo + Sepolia)
- Gateway: `0x1607C1368bc943130258318c91bBd8cFf3D063E6`
- BeefyClient: `0x2c780945beb1241fE9c645800110cb9C4bBbb639`

### Westend (Westend + Sepolia)
- Gateway: `0x9ed8b47bc3417e3bd0507adc06e56e2fa360a4e9`
- BeefyClient: `0x6DFaD3D73A28c48E4F4c616ECda80885b415283a`

## Troubleshooting

### Beacon state service not healthy

Check the logs:
```bash
docker compose logs beacon-state-service
```

Common issues:
- Beacon endpoint not reachable
- Incorrect fork versions (check your .env matches the network)

### Relayer failing to submit transactions

- Check private key is correctly stored in AWS Secrets Manager
- Verify AWS credentials in `.env`
- Check endpoint connectivity

### Gas estimation failures (ethereum relay)

- Ensure `snowbridge-gas-estimator` binary is available in the container
- Verify AssetHub and BridgeHub endpoints are correct

### Relayer not picking up messages

- Check that your relayer ID and total count are configured correctly in the config
- Multiple relayers coordinate using the `schedule` config to avoid duplicate submissions
- Ensure your endpoints are synced and not lagging

## Rewards

Relayers earn rewards for successfully delivering messages:

- **Polkadot → Ethereum** (`parachain-v3`): Rewards are paid in ETH on Ethereum
- **Ethereum → Polkadot** (`ethereum-v3`): Rewards are paid in DOT on AssetHub

To claim rewards, configure the `REWARD_ADDRESS` environment variable with your reward destination address.

The `reward` relayer service automatically claims accumulated rewards periodically.

## Monitoring

### CloudWatch Logging (AWS)

If running on AWS EC2, logs are automatically sent to CloudWatch when configured:

1. Attach an IAM role with CloudWatch Logs permissions to your EC2 instance
2. Set `AWS_REGION` in your `.env` file
3. Logs will appear in CloudWatch under `snowbridge/{environment}/`

### Local Logging

View logs locally:

```bash
# Follow all logs
docker compose logs -f

# Follow specific service
docker compose logs -f parachain-v3

# View last 100 lines
docker compose logs --tail 100 ethereum-v3
```

### Health Checks

```bash
# Beacon state service health
curl http://localhost:8080/health

# Check container status
docker compose ps
```

## Getting Help

- GitHub Issues: https://github.com/Snowfork/snowbridge/issues
- Discord: Join the Snowbridge Discord for community support
