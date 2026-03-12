#!/usr/bin/env bun
// Polkadot-JS script to generate a BEEFY checkpoint
// Usage: bun beefy-checkpoint.js [rpc-endpoint]
// Example: bun beefy-checkpoint.js wss://rpc.polkadot.io

import { ApiPromise, WsProvider } from "@polkadot/api";

const rpcEndpoint = process.argv[2] || "wss://rpc.polkadot.io";

async function generateBeefyCheckpoint() {
  console.log(`Connecting to ${rpcEndpoint}...`);

  const provider = new WsProvider(rpcEndpoint);
  const api = await ApiPromise.create({ provider });

  try {
    console.log("Fetching BEEFY checkpoint data...");

    // Get the finalized head
    const blockHash = await api.rpc.beefy.getFinalizedHead();
    const header = await api.rpc.chain.getHeader(blockHash);
    const apiAtBlock = await api.at(blockHash);

    // Fetch BEEFY authorities
    const authorities = await apiAtBlock.query.beefyMmrLeaf.beefyAuthorities();
    const nextAuthorities =
      await apiAtBlock.query.beefyMmrLeaf.beefyNextAuthorities();

    // Construct checkpoint
    const beefyCheckpoint = {
      blockNumber: header.number.toNumber(),
      blockHash: blockHash.toHex(),
      current: {
        id: authorities.id.toNumber(),
        root: authorities.keysetCommitment.toHex(),
        length: authorities.len.toNumber(),
      },
      next: {
        id: nextAuthorities.id.toNumber(),
        root: nextAuthorities.keysetCommitment.toHex(),
        length: nextAuthorities.len.toNumber(),
      },
    };

    console.log("\n=== BEEFY Checkpoint ===\n");
    console.log(JSON.stringify(beefyCheckpoint, null, 2));
  } catch (error) {
    console.error("Error generating BEEFY checkpoint:", error.message);
    process.exit(1);
  } finally {
    await api.disconnect();
  }
}

generateBeefyCheckpoint();
