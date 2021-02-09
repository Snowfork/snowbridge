// This script is to help with pulling in BEEFY data for generating fixtures for testing purposes.
// To use, run at least 2 relay chain nodes which have the BEEFY protocol active, eg:
//   polkadot --chain=rococo-local --tmp --ws-port=9944 --port=30444 --alice  --enable-offchain-indexing true
//   polkadot --chain=rococo-local --tmp --ws-port=9955 --port=30555 --bob  --enable-offchain-indexing true
// Then run this script to see output.
// Additionally, to get the addresses/public

const { ApiPromise, WsProvider } = require('@polkadot/api');
const WebSocket = require('ws');
const { ethereumEncode } = require("@polkadot/util-crypto");

const RELAY_CHAIN_RPC_ENDPOINT = 'ws://localhost:9944';
async function start() {
  const wsProvider = new WsProvider(RELAY_CHAIN_RPC_ENDPOINT);
  const api = await ApiPromise.create({
    provider: wsProvider,
    types: {
      SignedCommitment: {
        commitment: 'Commitment',
        signatures: 'Vec<Option<Signature>>'
      },
      Commitment: {
        payload: 'H256',
        block_number: 'BlockNumber',
        validator_set_id: 'ValidatorSetId'
      },
      ValidatorSetId: 'u64'
    }
  });

  const ws = new WebSocket('ws://localhost:9955');
  const currentAuthorities = await api.query.beefy.authorities();
  console.log({ currentAuthorities: currentAuthorities.map(a => ethereumEncode(a)) });

  const startSubscriptionRPC = {
    jsonrpc: '2.0',
    id: 1,
    method: "beefy_subscribeJustifications",
  }

  ws.on('open', function open() {
    ws.send(JSON.stringify(startSubscriptionRPC));
  });

  ws.on('message', function incoming(data) {
    jdata = JSON.parse(data);
    if (jdata && jdata.params && jdata.params.result) {
      console.log(api.createType('SignedCommitment', jdata.params.result).toString());
    }
  });

}

start();
