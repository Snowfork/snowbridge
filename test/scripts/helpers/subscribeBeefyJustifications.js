// This script is to help with pulling in BEEFY data for generating fixtures for testing purposes.
// To use, run at least 2 relay chain nodes which have the BEEFY protocol active, eg:
//   polkadot --chain=rococo-local --tmp --ws-port=9944 --port=30444 --alice  --enable-offchain-indexing true
//   polkadot --chain=rococo-local --tmp --ws-port=9955 --port=30555 --bob  --enable-offchain-indexing true
// Then run this script to see output.
// Additionally, to get the addresses/public

const { ApiPromise, WsProvider } = require('@polkadot/api');
const WebSocket = require('ws');
const { base58Decode, addressToEvm, secp256k1Expand, secp256k1Compress, decodeAddress, encodeAddress, ethereumEncode, blake2AsHex, keccakAsHex } = require("@polkadot/util-crypto");
const { hexToU8a, u8aToHex, u8aToU8a } = require("@polkadot/util");

const RELAY_CHAIN_RPC_ENDPOINT = 'ws://localhost:9944';

async function start() {
  const wsProvider = new WsProvider(RELAY_CHAIN_RPC_ENDPOINT);
  const api = await ApiPromise.create({
    provider: wsProvider,
    types: {
      SignedCommitment: {
        commitment: 'Commitment',
        signatures: 'Vec<Option<BeefySignature>>'
      },
      Commitment: {
        payload: 'H256',
        block_number: 'BlockNumber',
        validator_set_id: 'ValidatorSetId'
      },
      ValidatorSetId: 'u64',
      BeefySignature: '[u8; 65]',
      Authorities: 'Vec<[u8; 33]>',
      MMRStorageKey: {
        prefix: 'Vec<u8>',
        pos: 'u64'
      },
      MMRProof: {
        blockHash: 'BlockHash',
        leaf: 'Vec<u8>',
        proof: 'ActualMMRProof',
      },
      BlockHash: 'H256',
      ActualMMRProof: {
        /// The index of the leaf the proof is for.
        leaf_index: 'u64',
        /// Number of leaves in MMR, when the proof was generated.
        leaf_count: 'u64',
        /// Proof elements (hashes of siblings of inner nodes on the path to the leaf).
        items: 'Vec<Hash>',
      },
      MMRLeaf: {
        parent_number_and_hash: 'ParentNumberAndHash',
        parachainHeads: 'H256',
        beefyNextAuthoritySet: 'BeefyNextAuthoritySet',
      },
      ParentNumberAndHash: {
        parentNumber: 'ParentNumber',
        hash: '[u8; 32]'
      },
      // TODO: The MMRLeaf is a Vec<u8>, so double-scale encoded which messes this first variable up.
      // Should fix
      ParentNumber: { idk: '[u8; 2]', blockNumber: 'u32' },
      BeefyNextAuthoritySet: {
        id: 'u64',
        /// Number of validators in the set.
        len: 'u32',
        /// Merkle Root Hash build from BEEFY uncompressed AuthorityIds.
        root: 'H256',
      }
    },
    rpc: {
      beefy: {
        subscribeJustifications: {
          alias: ['beefy_subscribeJustifications', 'beefy_unsubscribeJustifications'],
          params: [],
          type: 'SignedCommitment',
          pubsub: [
            'justifications',
            'subscribeJustifications',
            'unsubscribeJustifications'
          ],
        }
      },
      mmr: {
        generateProof: {
          alias: ['mmr_generateProof'],
          params: [{
            name: 'leaf_index',
            type: 'u64'
          }],
          type: 'MMRProof'
        }
      }
    }
  });

  await getAuthorities(api);
  await subscribeJustifications(api);
}

async function getAuthorities(api) {
  const authoritiesResponse = await api.query.beefy.authorities();
  let authorities = api.createType('Authorities', authoritiesResponse);

  // Currently, there is a bug in the Javascript decoding of the authority which results in it missing the last byte
  // For now, we just replace this with the correct ones for testing purposes, but when integrated into the relayer
  // we'll need to ensure the authorities are decoded correctly from the actual API. See the console.log for example.
  const correctAuthorityAlice = '0x020a1091341fe5664bfa1782d5e04779689068c916b04cb365ec3153755684d9a1';
  const correctAuthorityBob = '0x0390084fdbf27d2b79d26a4f13f0ccd982cb755a661969143c37cbc49ef5b91f27';
  authoritiesCorrect = [correctAuthorityAlice, correctAuthorityBob];
  console.log({
    authoritiesWrong: authorities.map(a => u8aToHex(a)),
    authoritiesCorrect,
  })
  authorities = authoritiesCorrect;

  const authoritiesEthereum = authorities.map(a => ethereumEncode(a));
  console.log({
    authoritiesEthereum
  });

  return authoritiesEthereum;
}

async function subscribeJustifications(api) {
  api.rpc.beefy.subscribeJustifications(justification => {
    const commitment = justification.commitment;
    const commitmentBytes = commitment.toHex();
    const hashedCommitment = blake2AsHex(commitmentBytes, 256);
    console.log({ justification: justification.toString(), commitmentBytes, hashedCommitment });
    getLatestMMRInJustification(justification, api)
  });
}

async function getLatestMMRInJustification(justification, api) {
  const blockNumber = justification.commitment.block_number.toString();
  const mmrRoot = justification.commitment.payload.toString();
  console.log({
    blockNumber,
    mmrRoot
  });

  latestMMRLeaf = getMMRLeafForBlock(blockNumber, api)

  // TODO Extract para_heads from MMR
  // TODO Get proof and para_head of our parachain in para_heads
}

async function getMMRLeafForBlock(blockNumber, api) {
  const mmrProof = await api.rpc.mmr.generateProof(blockNumber);
  console.log({ mmrProof: mmrProof.toString() });

  decodedLeaf = api.createType('MMRLeaf', mmrProof.leaf.toHex());
  console.log({ decodedLeaf: decodedLeaf.toString() })
}

start();
