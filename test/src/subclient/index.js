let { ApiPromise, WsProvider } = require('@polkadot/api');
let { Keyring } = require('@polkadot/api');
const { default: BigNumber } = require('bignumber.js');

class SubClient {

  constructor(endpoint) {
    this.endpoint = endpoint;
    this.api = null;
    this.keyring = null;
  }

  async connect() {
    const provider = new WsProvider(this.endpoint);
    this.api = await ApiPromise.create({
      provider,
      types: {
        "Address": "MultiAddress",
        "LookupSource": "MultiAddress",
        "ChannelId": {
          "_enum": {
            "Basic": null,
            "Incentivized": null
          }
        },
        "MessageNonce": "u64",
        "MessageId": {
          "channelId": "ChannelId",
          "nonce": "u64"
        },
        "Message": {
          "data": "Vec<u8>",
          "proof": "Proof"
        },
        "Proof": {
          "blockHash": "H256",
          "txIndex": "u32",
          "data": "(Vec<Vec<u8>>, Vec<Vec<u8>>)"
        },
        "EthereumHeader": {
          "parentHash": "H256",
          "timestamp": "u64",
          "number": "u64",
          "author": "H160",
          "transactionsRoot": "H256",
          "ommersHash": "H256",
          "extraData": "Vec<u8>",
          "stateRoot": "H256",
          "receiptsRoot": "H256",
          "logBloom": "Bloom",
          "gasUsed": "U256",
          "gasLimit": "U256",
          "difficulty": "U256",
          "seal": "Vec<Vec<u8>>"
        },
        "EthashProofData": {
          "dagNodes": "[H512; 2]",
          "proof": "Vec<H128>"
        },
        "Bloom": {
          "_": "[u8; 256]"
        },
        "PruningRange": {
          "oldestUnprunedBlock": "u64",
          "oldestBlockToKeep": "u64"
        },
        "AssetId": {
          "_enum": {
            "ETH": null,
            "Token": "H160"
          }
        },
        "InboundChannelData": {
          "nonce": "u64"
        },
        "OutboundChannelData": {
          "nonce": "u64"
        }
      }
    })

    this.keyring = new Keyring({ type: 'sr25519' });
    this.alice = this.keyring.addFromUri('//Alice', { name: 'Alice' });

  }

  async queryAccountBalance(accountId, assetId) {
    let balance = await this.api.query.assets.balances(assetId, accountId);
    return BigNumber(balance.toBigInt())
  }

  async burnETH(account, recipient, amount) {
    const txHash = await this.api.tx.eth.burn(0, recipient, amount).signAndSend(account);
  }

  async burnERC20(account, assetId, recipient, amount) {
    const txHash = await this.api.tx.erc20.burn(0, assetId, recipient, amount).signAndSend(account);
  }


}

module.exports.SubClient = SubClient;
