import type { RegistryTypes, OverrideModuleType } from '@polkadot/types/types';

export const types: RegistryTypes = {
  Address: "MultiAddress",
  LookupSource: "MultiAddress",
  ChannelId: {
    _enum: ["Basic", "Incentivized"]
  },
  MessageNonce: "u64",
  MessageId: {
    channelId: "ChannelId",
    nonce: "u64"
  },
  Message: {
    data: "Vec<u8>",
    proof: "Proof"
  },
  Proof: {
    blockHash: "H256",
    txIndex: "u32",
    data: "(Vec<Vec<u8>>, Vec<Vec<u8>>)"
  },
  EthereumHeaderId: {
    number: "u64",
    hash: "H256"
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
    seal: "Vec<Vec<u8>>"
  },
  StoredHeader: {
    submitter: "Option<AccountId>",
    header: "EthereumHeader",
    totalDifficulty: "U256"
  },
  EthashProofData: {
    dagNodes: "[H512; 2]",
    proof: "Vec<H128>"
  },
  Bloom: "[u8; 256]",
  PruningRange: {
    oldestUnprunedBlock: "u64",
    oldestBlockToKeep: "u64"
  },
  AssetId: {
    _enum: {
      ETH: null,
      Token: "H160"
    }
  },
  InboundChannelData: {
    nonce: "u64"
  },
  OutboundChannelData: {
    nonce: "u64"
  },
  IndexingPrefix: '[u8; 11]',
  AuxiliaryDigestItem: {
    isCommitment: 'bool',
    commitment: 'Commitment'
  },
  Commitment: '(ChannelId, H256)',
  OffchainCommitmentKey: '(IndexingPrefix, ChannelId, H256)',
  ChannelMessage: {
    target: 'H160',
    nonce: 'u64',
    payload: 'Vec<u8>',
  },
  ChannelMessages: 'Vec<ChannelMessage>'
}
