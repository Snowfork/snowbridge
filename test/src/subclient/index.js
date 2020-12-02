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
                }
            }
        })

        this.keyring = new Keyring({ type: 'sr25519' });
        this.alice = this.keyring.addFromUri('//Alice', { name: 'Alice' });

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


}

module.exports.SubClient = SubClient;

