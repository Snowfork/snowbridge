# Docker Deployment Guide

This guide explains how to deploy Snowbridge relayers using Docker Compose.

## Prerequisites

- Docker and Docker Compose installed
- AWS account with Secrets Manager access (for private key storage)
- RPC endpoints for:
  - Ethereum execution layer (WebSocket)
  - Ethereum beacon chain (HTTP)
  - Polkadot relay chain (WebSocket)
  - BridgeHub parachain (WebSocket)
  - AssetHub parachain (WebSocket, for execution relay gas estimation)

## Quick Start

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
   - AWS credentials and secret key IDs
   - (Mainnet only) Chainalysis API key for OFAC compliance

3. **Start the relayers:**
   ```bash
   docker compose up -d
   ```

## Architecture

The Docker Compose setup runs 6 relayer services:

| Service | Description | Keys Required |
|---------|-------------|---------------|
| `beacon-state-service` | Caches beacon state proofs | None |
| `beacon` | Relays Ethereum beacon headers to Polkadot | Substrate |
| `execution` | Relays Ethereum messages to Polkadot | Substrate |
| `beefy` | Relays BEEFY commitments to Ethereum | Ethereum |
| `parachain` | Relays Polkadot messages to Ethereum | Ethereum + Substrate |
| `reward` | Processes relayer rewards | Substrate |

### Service Dependencies

```
beacon-state-service (starts first, health checked)
    ├── beacon
    ├── execution
    ├── parachain
    └── reward

beefy (independent)
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

### Private Keys (AWS Secrets Manager)

Private keys are stored in AWS Secrets Manager and referenced by ID:

```bash
# Pattern: {network}/{relay-name}
BEACON_RELAY_SUBSTRATE_KEY_ID=mainnet/beacon-relay
EXECUTION_RELAY_SUBSTRATE_KEY_ID=mainnet/asset-hub-ethereum-relay-v2
```

Create secrets in AWS Secrets Manager containing the raw private key strings.

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
docker compose logs -f beacon
```

### Stop relayers

```bash
docker compose down
```

### Restart a specific relayer

```bash
docker compose restart execution
```

### Check health

```bash
# Beacon state service health
curl http://localhost:8080/health
```

## Volumes

The setup creates persistent volumes for:
- `beacon-state-data` - Beacon state service cache and persistence
- `beacon-data` - Beacon relay local datastore

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

### Gas estimation failures (execution relay)

- Ensure `snowbridge-gas-estimator` binary is available in the container
- Verify AssetHub and BridgeHub endpoints are correct
