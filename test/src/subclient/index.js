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
                "AppId": "[u8; 20]",
                "Message": {
                  "payload": "Vec<u8>",
                  "verification": "VerificationInput"
                },
                "VerificationInput": {
                  "_enum": {
                    "Basic": "VerificationBasic",
                    "None": null
                  }
                },
                "VerificationBasic": {
                  "blockNumber": "u64",
                  "eventIndex": "u32"
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
                "Bloom": {
                  "_": "[u8; 256]"
                },
                "AssetId": {
                  "_enum": {
                    "ETH": null,
                    "Token": "H160"
                  }
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
        const txHash = await this.api.tx.eth.burn(recipient, amount).signAndSend(account);
    }

    async burnERC20(account, assetId, recipient, amount) {
        const txHash = await this.api.tx.erc20.burn(assetId, recipient, amount).signAndSend(account);
    }


}

module.exports.SubClient = SubClient;

