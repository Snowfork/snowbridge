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
    },
    rpc: {
      beefy: {
        subscribeJustifications: {
          alias: ['beefy_subscribeJustifications', 'beefy_unsubscribeJustifications'],
          params: [],
          type: 'SignedCommitment',
          method: _ => console.log("qq"),
          pubsub: [
            'justifications',
            'subscribeJustifications',
            'unsubscribeJustifications'
          ],
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
  // Print relevant API for reference:
  console.log({ mmr: Object.keys(api.query.mmr) })
  console.log({ beefy: Object.keys(api.query.beefy) })
  console.log({ mmrLeaf: Object.keys(api.query.mmrLeaf) })
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

async function getMMRLeafForBlock(bblockNumberlock, api) {
  // TODO query offchain storage for that MMRLeaf
  // TODO print that MMRLeaf
}

start();
