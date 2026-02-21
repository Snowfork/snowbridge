# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Snowbridge is a trustless bridge between Polkadot and Ethereum. This repository contains the bridge components (contracts, relayer, tests, etc.) while the parachain and pallets are in a separate fork of polkadot-sdk.

## Development Environment Setup

### Nix Flakes (Required)
The project uses Nix flakes for reproducible development:

```bash
# Activate developer shell
nix develop

# Run once for initialization
scripts/init.sh
```

### Editor Integration
For VS Code, run within the interactive shell:
```bash
nix develop
code .
```

To preserve existing shell:
```bash
nix develop --command $SHELL
```

For automatic activation with direnv:
```bash
cp .envrc.example .envrc
direnv allow
```

## Build and Test Commands

### Contracts (Foundry)
```bash
cd contracts
forge build              # Build contracts
forge test               # Run tests
forge coverage           # Coverage reports
```

### Relayer (Go/Mage)
```bash
cd relayer
mage build              # Build relayer
mage test               # Run unit and integration tests
mage lint               # Run revive linter

# Generate contract bindings after contract changes
go generate ./...
```

### Web UI (Turborepo)
```bash
cd web
pnpm install            # Install dependencies
turbo run build         # Build all packages
turbo run test          # Run tests
turbo run lint          # Run linters
turbo run format        # Format code
turbo run coverage      # Run coverage
```

### SP1 Beefy Client
```bash
cd sp1-beefy-client
cargo prove build        # Build ZK program
cd script
cargo run -- [command]  # Generate proofs
```

### Smoke Tests (Rust)
```bash
cd smoketest
./make-bindings.sh      # Generate Rust bindings
cargo test --test [test_name]  # Run specific tests
```

## Architecture Overview

### Core Components

1. **Gateway Contract** (`contracts/`)
   - Central hub for cross-chain messaging
   - Modular design with Verification and Assets libraries
   - ERC-1967 upgradeable pattern
   - Located in `contracts/src/Gateway.sol`

2. **Relayer Service** (`relayer/`)
   - Go-based service streaming transactions
   - Multiple relayer instances with coordinated scheduling
   - Generates contract bindings dynamically
   - Configuration-based for different networks

3. **SP1 Beefy Client** (`sp1-beefy-client/`)
   - Zero-knowledge verification for Beefy consensus
   - Replaces expensive on-chain verification with cheap proof verification
   - Generates proofs for: submitInitial, commitPrevRandao, submitFinal, submitFiatShamir

4. **Web UI** (`web/`)
   - TypeScript monorepo using Turborepo
   - Packages: API, contract types, operations, registry
   - End-to-end testing capabilities

### Message Flow

1. **Ethereum → Polkadot**:
   - Users call Gateway contract
   - Relayers package messages and submit to BridgeHub
   - Assets locked in Agent contracts, minted on parachain

2. **Polkadot → Ethereum**:
   - Parachain sends messages via BridgeHub
   - BeefyClient verifies consensus proofs
   - Relayers execute on Ethereum via Agent contracts

### Key Dependencies

- **Rust**: Substrate runtime, SP1 zkVM
- **Go**: Ethereum clients, AWS SDK, Substrate RPC
- **Solidity**: Foundry framework, OpenZeppelin contracts
- **TypeScript**: Substrate client, contract bindings

## Testing Strategy

### Unit Tests
- Contracts: Foundry test suite
- Relayer: Go unit tests with revive linting
- Web: Jest/Playwright tests

### Integration Tests
- Smoke tests running against E2E stack
- Local testnet setup in `web/packages/test`
- Multi-relayer coordination tests

### Test Data
- BEEFY fixtures from `/tmp/snowbridge/beefy-relay.log`
- Dynamic contract bindings generation required
- E2E stack auto-generates suitable configurations

## Current Context

**Branch**: `ron/beefy-sp1`
- Recent work on SP1 integration with Beefy consensus
- Modified files: `relayer/contracts/beefy_client.go`, `relayer/contracts/gateway.go`
- Untracked `target/` directory from build artifacts

## Configuration Notes

### Relayer Coordination
Multiple relayer instances require coordination:
- Each relayer has unique ID and total count
- Sleep interval for nonce checking
- Configuration files: `execution-relay-asset-hub-{id}.json`

### Environment Variables
Set by E2E stack for local development:
- Blockchain RPC endpoints
- Contract addresses
- AWS credentials (for production)

### Cross-Chain Governance
Gateway upgrades initiated via relaychain governance
- Requires proper permissions on Polkadot side
- Affects bridge availability during upgrades