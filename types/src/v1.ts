import type { RegistryTypes, OverrideModuleType, AliasDefinition } from "@polkadot/types/types";

export const alias: AliasDefinition = {
  dispatch: {
    MessageId: "DispatchMessageId"
  }
};

export const types: RegistryTypes = {
  Address: "MultiAddress",
  LookupSource: "MultiAddress",
  DispatchMessageId: {
    channelId: "ChannelId",
    nonce: "u64",
  },
  ChannelId: {
    _enum: ["Basic", "Incentivized"],
  },
  MessageNonce: "u64",
  Message: {
    data: "Vec<u8>",
    proof: "Proof",
  },
  Proof: {
    blockHash: "H256",
    txIndex: "u32",
    data: "(Vec<Vec<u8>>, Vec<Vec<u8>>)",
  },
  EthereumHeaderId: {
    number: "u64",
    hash: "H256",
  },
  EthereumHeader: {
    parentHash: "H256",
    timestamp: "u64",
    number: "u64",
    author: "H160",
    transactionsRoot: "H256",
    ommersHash: "H256",
    extraData: "Vec<u8>",
    stateRoot: "H256",
    receiptsRoot: "H256",
    logBloom: "Bloom",
    gasUsed: "U256",
    gasLimit: "U256",
    difficulty: "U256",
    seal: "Vec<Vec<u8>>",
    baseFee: "Option<U256>",
  },
  StoredHeader: {
    submitter: "Option<AccountId>",
    header: "EthereumHeader",
    totalDifficulty: "U256",
    finalized: "bool",
  },
  EthashProofData: {
    dagNodes: "[H512; 2]",
    proof: "Vec<H128>",
  },
  Bloom: "[u8; 256]",
  PruningRange: {
    oldestUnprunedBlock: "u64",
    oldestBlockToKeep: "u64",
  },
  EthereumDifficultyConfig: {
    byzantiumForkBlock: "u64",
    constantinopleForkBlock: "u64",
    muirGlacierForkBlock: "u64",
    londonForkBlock: "u64"
  },
  AssetId: {
    _enum: {
      ETH: null,
      Token: "H160",
    },
  },
  TokenId: "u128",
  TokenData: {
    tokenContract: "H160",
    tokenId: "U256",
  },
  TokenInfoOf: {
    owner: "AccountId",
    metadata: "Vec<u8>",
    data: "TokenData",
  },
};
