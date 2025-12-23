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
    },
    westend_sepolia: {
        name: "westend_sepolia",
        ethChainId: 11155111,
        beaconApiUrl: "https://lodestar-sepolia.chainsafe.io",
        ethereumChains: {
            "11155111": "https://ethereum-sepolia-rpc.publicnode.com",
        },
        relaychainUrl: "wss://westend-rpc.n.dwellir.com",
        parachains: {
            "1000": "wss://asset-hub-westend-rpc.n.dwellir.com",
            "1002": "wss://bridge-hub-westend-rpc.n.dwellir.com",
        },
        gatewayContract: "0x9ed8b47bc3417e3bd0507adc06e56e2fa360a4e9",
        beefyContract: "0x6DFaD3D73A28c48E4F4c616ECda80885b415283a",
        assetHubParaId: 1000,
        bridgeHubParaId: 1002,
        indexerGraphQlUrl:
            "https://snowbridge.squids.live/snowbridge-subsquid-westend@v1/api/graphql",
    },
}
