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
        const provider = new WsProvider('ws://127.0.0.1:9944');
        this.api = await ApiPromise.create({
            provider,
            types: {
                Address: 'AccountId',
                LookupSource: 'AccountId',
                AppId: '[u8; 20]',
                Message: {
                    payload: 'Vec<u8>',
                    verification: 'VerificationInput'
                },
                VerificationInput: {
                    _enum: {
                        Basic: 'VerificationBasic',
                        None: null
                    }
                },
                VerificationBasic: {
                    blockNumber: 'u64',
                    eventIndex: 'u32'
                },
                TokenId: 'H160',
                BridgedAssetId: 'H160',
                AssetAccountData: {
                    free: 'U256'
                },
                EthereumHeader: {
                    parentHash: 'H256',
                    timestamp: 'u64',
                    number: 'u64',
                    author: 'H160',
                    transactionsRoot: 'H256',
                    ommersHash: 'H256',
                    extraData: 'Vec<u8>',
                    stateRoot: 'H256',
                    receiptsRoot: 'H256',
                    logsBloom: 'Bloom',
                    gasUsed: 'U256',
                    gasLimit: 'U256',
                    difficulty: 'U256',
                    seal: 'Vec<Vec<u8>>'
                },
                Bloom: {
                    _: '[u8; 256]'
                }
            }
        })

        this.keyring = new Keyring({ type: 'sr25519' });
        this.alice = this.keyring.addFromUri('//Alice', { name: 'Alice' });
        this.relay = this.keyring.addFromUri('//Relay', { name: 'Relay' });
    }

    async queryAccountBalance(accountId, assetId) {
        let accountData = await this.api.query.asset.account(assetId, accountId);
        if (accountData && accountData.free) {
            return BigNumber(accountData.free.toBigInt())
        }
        return null
    }

    async burnETH(account, recipient, amount) {
        const txHash = await this.api.tx.eth.burn(recipient, amount).signAndSend(account);
    }

    async burnERC20(account, assetId, recipient, amount) {
        const txHash = await this.api.tx.erc20.burn(assetId, recipient, amount).signAndSend(account);
    }

    async importEthHeader(account, orderedFields) {
        const txHash = await this.api.tx.verifier.importHeader(orderedFields).signAndSend(account);
    }
}

module.exports.SubClient = SubClient;

