// Polkadot-JS script to generate a BEEFY checkpoint

let beefyBlock = 20733663;
let blockHash = await api.rpc.chain.getBlockHash(beefyBlock);
let apiAtBlock = await api.at(blockHash);

let authorities = await apiAtBlock.query.beefyMmrLeaf.beefyAuthorities();
let nextAuthorities =
  await apiAtBlock.query.beefyMmrLeaf.beefyNextAuthorities();

let beefyCheckpoint = {
  startBlock: beefyBlock,
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

console.log(JSON.stringify(beefyCheckpoint, null, 2));
