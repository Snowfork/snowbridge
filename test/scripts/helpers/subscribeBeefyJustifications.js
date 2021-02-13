// This script is to help with pulling in BEEFY data for generating fixtures for testing purposes.
// To use, run at least 2 relay chain nodes which have the BEEFY protocol active, eg:
//   polkadot --chain=rococo-local --tmp --ws-port=9944 --port=30444 --alice  --enable-offchain-indexing true
//   polkadot --chain=rococo-local --tmp --ws-port=9955 --port=30555 --bob  --enable-offchain-indexing true
// Then run this script to see output.
// Additionally, to get the addresses/public

const { ApiPromise, WsProvider } = require('@polkadot/api');
const WebSocket = require('ws');
const { base58Decode, addressToEvm, secp256k1Expand, secp256k1Compress, decodeAddress, ethereumEncode, blake2AsHex, keccakAsHex } = require("@polkadot/util-crypto");
const { u8aToHex, u8aToU8a } = require("@polkadot/util");

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
    }
  });

  const ws = new WebSocket('ws://localhost:9955');
  const currentAuthorities = await api.query.beefy.authorities();

  const currentAuthoritiesString = currentAuthorities.map(a => a.toString());
  const currentAuthoritiesu8aAddress = currentAuthorities.map(a => u8aToU8a(a));
  const currentAuthoritiessecp256k1Expand = currentAuthoritiesu8aAddress.map(a => ethereumEncode(a));
  const currentAuthoritiesDecoded = currentAuthoritiesString.map(a => u8aToHex(decodeAddress(a)));
  // const currentAuthoritiesExpanded = currentAuthoritiesDecoded.map(a => secp256k1Compress(a));
  // const currentAuthoritiesETHEncoded = currentAuthorities.map(a => ethereumEncode(a));

  console.log({ currentAuthoritiesString, currentAuthoritiesu8aAddress, currentAuthoritiessecp256k1Expand });
  console.log({ currentAuthorities: currentAuthorities.map(a => u8aToHex(decodeAddress(a))) });
  console.log({ currentAuthoritiesEth: currentAuthorities.map(a => ethereumEncode(a)) });
  console.log({ currentAuthoritiesEthKec: currentAuthorities.map(a => keccakAsHex(ethereumEncode(a))) });

  const startSubscriptionRPC = {
    jsonrpc: '2.0',
    id: 1,
    method: "beefy_subscribeJustifications",
  }

  ws.on('open', function open() {
    ws.send(JSON.stringify(startSubscriptionRPC));
  });

  const getMethods = (obj) => {
    let properties = new Set()
    let currentObj = obj
    do {
      Object.getOwnPropertyNames(currentObj).map(item => properties.add(item))
    } while ((currentObj = Object.getPrototypeOf(currentObj)))
    return [...properties.keys()].filter(item => typeof obj[item] === 'function')
  }

  ws.on('message', function incoming(data) {
    jdata = JSON.parse(data);
    if (jdata && jdata.params && jdata.params.result) {
      console.log(typeof jdata.params.result)
      const signedCommitment = api.createType('SignedCommitment', jdata.params.result);
      const signedCommitmentJSON = api.createType('SignedCommitment', jdata.params.result).toJSON();
      const commitment = signedCommitment.commitment;
      const commitmentBytes = commitment.toHex();
      const hashedCommitment = blake2AsHex(commitmentBytes, 256);
      console.log({ signedCommitmentJSON, commitmentBytes, hashedCommitment });
    }
  });

}

start();
