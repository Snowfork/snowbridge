import { Environment } from "@snowbridge/base-types"

export function environmentFor(
    env: "polkadot_mainnet" | "westend_sepolia" | "paseo_sepolia" | (string & {}),
): Environment {
    if (!(env in SNOWBRIDGE_ENV)) throw Error(`Unknown env '${env}'`)
    return SNOWBRIDGE_ENV[env]
}

const SNOWBRIDGE_ENV: { [env: string]: Environment } = {
    local_e2e: {
        name: "local_e2e",
        ethChainId: 11155111,
        beaconApiUrl: "http://127.0.0.1:9596",
        ethereumChains: {
            "11155111": "ws://127.0.0.1:8546",
        },
        relaychainUrl: "ws://127.0.0.1:9944",
        parachains: {
            "1000": "ws://127.0.0.1:12144",
            "1002": "ws://127.0.0.1:11144",
            "2000": "ws://127.0.0.1:13144",
        },
        gatewayContract: "0xb1185ede04202fe62d38f5db72f71e38ff3e8305",
        beefyContract: "0x83428c7db9815f482a39a1715684dcf755021997",
        assetHubParaId: 1000,
        bridgeHubParaId: 1002,
        indexerGraphQlUrl: "http://127.0.0.1/does/not/exist",
    },
    paseo_sepolia: {
        name: "paseo_sepolia",
        ethChainId: 11155111,
        beaconApiUrl: "https://lodestar-sepolia.chainsafe.io",
        ethereumChains: {
            "11155111": "https://ethereum-sepolia-rpc.publicnode.com",
        },
        relaychainUrl: "wss://paseo-rpc.dwellir.com",
        parachains: {
            "1000": "wss://asset-hub-paseo-rpc.dwellir.com",
            "1002": "wss://bridge-hub-paseo.dotters.network",
            "3369": "wss://paseo-muse-rpc.polkadot.io",
            "2043": `wss://parachain-testnet-rpc.origin-trail.network`,
        },
        gatewayContract: "0x1607C1368bc943130258318c91bBd8cFf3D063E6",
        beefyContract: "0x2c780945beb1241fE9c645800110cb9C4bBbb639",
        assetHubParaId: 1000,
        bridgeHubParaId: 1002,
        indexerGraphQlUrl:
            "https://snowbridge.squids.live/snowbridge-subsquid-paseo@v1/api/graphql",
        metadataOverrides: {
            // Change the name of TRAC
            "0xef32abea56beff54f61da319a7311098d6fbcea9": {
                name: "OriginTrail TRAC",
                symbol: "TRAC",
            },
        },
    },
    polkadot_mainnet: {
        name: "polkadot_mainnet",
        ethChainId: 1,
        beaconApiUrl: "https://lodestar-mainnet.chainsafe.io",
        ethereumChains: {
            "1": "https://ethereum-rpc.publicnode.com",
            "1284": "https://rpc.api.moonbeam.network",
            "8453": "https://base-rpc.publicnode.com",
        },
        relaychainUrl: "https://polkadot-rpc.n.dwellir.com",
        parachains: {
            "1000": "wss://asset-hub-polkadot-rpc.n.dwellir.com",
            "1002": "https://bridge-hub-polkadot-rpc.n.dwellir.com",
            "3369": "wss://polkadot-mythos-rpc.polkadot.io",
            "2034": "wss://hydration-rpc.n.dwellir.com",
            "2030": "wss://bifrost-polkadot.ibp.network",
            "2004": "wss://moonbeam.ibp.network",
            "2000": "wss://acala-rpc-0.aca-api.network",
            "2043": "wss://parachain-rpc.origin-trail.network",
            // TODO: Add back in jampton once we have an indexer in place.
            //"3397": "wss://rpc.jamton.network",
        },
        gatewayContract: "0x27ca963c279c93801941e1eb8799c23f407d68e7",
        beefyContract: "0x1817874feAb3ce053d0F40AbC23870DB35C2AFfc",
        assetHubParaId: 1000,
        bridgeHubParaId: 1002,
        indexerGraphQlUrl:
            "https://snowbridge.squids.live/snowbridge-subsquid-polkadot:production/api/graphql",
        kusama: {
            assetHubParaId: 1000,
            bridgeHubParaId: 1002,
            parachains: {
                "1000": "wss://asset-hub-kusama-rpc.n.dwellir.com",
                "1002": "https://bridge-hub-kusama-rpc.n.dwellir.com",
            },
        },
        precompiles: {
            // Add override for mythos token and add precompile for moonbeam
            "2004": "0x000000000000000000000000000000000000081a",
        },
        metadataOverrides: {
            // Change the name of TRAC
            "0xaa7a9ca87d3694b5755f213b5d04094b8d0f0a6f": {
                name: "OriginTrail TRAC",
            },
        },
        l2Bridge: {
            acrossAPIUrl: "https://app.across.to/api",
            l1AdapterAddress: "0xA44626f738e4369f1774b84Fb28Fd10f5a73a76f",
            l1HandlerAddress: "0x924a9f036260DdD5808007E1AA95f08eD08aA569",
            l1FeeTokenAddress: "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2",
            l1SwapQuoterAddress: "0x61fFE014bA17989E743c5F6cB21bF9697530B21e",
            l1SwapRouterAddress: "0xE592427A0AEce92De3Edee1F18E0157C05861564",
            l2Chains: {
                "8453": {
                    adapterAddress: "0xCd5d2c665E3AC84bF5c67FE7a0C48748dA40db2F",
                    feeTokenAddress: "0x4200000000000000000000000000000000000006",
                    swapRoutes: [
                        // WETH
                        {
                            inputToken: "0x4200000000000000000000000000000000000006",
                            outputToken: "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2",
                            swapFee: 0,
                        },
                        // USDC
                        {
                            inputToken: "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913",
                            outputToken: "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
                            swapFee: 500,
                        },
                    ],
                },
            },
        },
    },
    westend_sepolia: {
        name: "westend_sepolia",
        ethChainId: 11155111,
        beaconApiUrl: "https://lodestar-sepolia.chainsafe.io",
        ethereumChains: {
            "11155111": "https://ethereum-sepolia-rpc.publicnode.com",
            "84532": "https://base-sepolia-rpc.publicnode.com",
        },
        relaychainUrl: "wss://westend-rpc.n.dwellir.com",
        parachains: {
            "1000": "wss://asset-hub-westend-rpc.n.dwellir.com",
            "1002": "wss://bridge-hub-westend-rpc.n.dwellir.com",
        },
        gatewayContract: "0x9ed8b47bc3417e3bd0507adc06e56e2fa360a4e9",
        beefyContract: "0xA04460B1D8bBef33F54edB2C3115e3E4D41237A6",
        assetHubParaId: 1000,
        bridgeHubParaId: 1002,
        indexerGraphQlUrl:
            "https://snowbridge.squids.live/snowbridge-subsquid-westend@v1/api/graphql",
        l2Bridge: {
            acrossAPIUrl: "https://testnet.across.to/api",
            l1AdapterAddress: "0x33Fe409089c8AAd8Af119a8Dacd1ea6be3A3cbd5",
            l1HandlerAddress: "0x924a9f036260DdD5808007E1AA95f08eD08aA569",
            l1FeeTokenAddress: "0xfFf9976782d46CC05630D1f6eBAb18b2324d6B14",
            l1SwapRouterAddress: "0x3bFA4769FB09eefC5a80d6E87c3B9C650f7Ae48E",
            l1SwapQuoterAddress: "0xEd1f6473345F45b75F8179591dd5bA1888cf2FB3",
            l2Chains: {
                "84532": {
                    adapterAddress: "0xf06939613A3838Af11104c898758220dB9093679",
                    feeTokenAddress: "0x4200000000000000000000000000000000000006",
                    swapRoutes: [
                        // WETH
                        {
                            inputToken: "0x4200000000000000000000000000000000000006",
                            outputToken: "0xfff9976782d46cc05630d1f6ebab18b2324d6b14",
                            swapFee: 0,
                        },
                        // USDC
                        {
                            inputToken: "0x036cbd53842c5426634e7929541ec2318f3dcf7e",
                            outputToken: "0x1c7d4b196cb0c7b01d743fbc6116a902379c7238",
                            swapFee: 500,
                        },
                    ],
                },
            },
        },
    },
}
