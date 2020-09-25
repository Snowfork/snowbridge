let { ApiPromise, WsProvider } = require('@polkadot/api');

class SubClient {

    constructor(endpoint) {
        this.endpoint = endpoint
        this.api = null
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
                }
            }
        })
    }

    async queryAssetAccountData(assetId, accountId) {
        return this.api.query.asset.account(assetId, accountId);
    }

}

module.exports.SubClient = SubClient;

