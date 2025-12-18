/** @deprecated Use `Environment` from base-types */
export type Config = {
    BEACON_HTTP_API: string
    ETHEREUM_CHAINS: { [chain: string]: string }
    RELAY_CHAIN_URL: string
    GATEWAY_CONTRACT: string
    BEEFY_CONTRACT: string
    ASSET_HUB_PARAID: number
    BRIDGE_HUB_PARAID: number
    PRIMARY_GOVERNANCE_CHANNEL_ID: string
    SECONDARY_GOVERNANCE_CHANNEL_ID: string
    RELAYERS: Relayer[]
    PARACHAINS: { [paraId: string]: string }
    GRAPHQL_API_URL: string
    TO_MONITOR_PARACHAINS?: number[]
}

export type KusamaConfig = {
    ASSET_HUB_PARAID: number
    BRIDGE_HUB_PARAID: number
    PARACHAINS: { [paraId: string]: string }
}

export type SourceType = "substrate" | "ethereum"
export type Relayer = { name: string; account: string; type: SourceType; balance?: bigint }

/** @deprecated Use `Environment` from base-types */
export type SnowbridgeEnvironment = {
    config: Config
    kusamaConfig?: KusamaConfig
    name: string
    ethChainId: number
}

/** @deprecated Use `environmentFor` from asset-registry */
export const SNOWBRIDGE_ENV: { [id: string]: SnowbridgeEnvironment } = {
    local_e2e: {
        name: "local_e2e",
        ethChainId: 11155111,
        config: {
            BEACON_HTTP_API: "http://127.0.0.1:9596",
            ETHEREUM_CHAINS: {
                "11155111": "ws://127.0.0.1:8546",
            },
            RELAY_CHAIN_URL: "ws://127.0.0.1:9944",
            PARACHAINS: {
                "1000": "ws://127.0.0.1:12144",
                "1002": "ws://127.0.0.1:11144",
                "2000": "ws://127.0.0.1:13144",
            },
            GATEWAY_CONTRACT: "0xb1185ede04202fe62d38f5db72f71e38ff3e8305",
            BEEFY_CONTRACT: "0x83428c7db9815f482a39a1715684dcf755021997",
            ASSET_HUB_PARAID: 1000,
            BRIDGE_HUB_PARAID: 1002,
            PRIMARY_GOVERNANCE_CHANNEL_ID:
                "0x0000000000000000000000000000000000000000000000000000000000000001",
            SECONDARY_GOVERNANCE_CHANNEL_ID:
                "0x0000000000000000000000000000000000000000000000000000000000000002",
            RELAYERS: [
                {
                    name: "beacon",
                    account: "5GWFwdZb6JyU46e6ZiLxjGxogAHe8SenX76btfq8vGNAaq8c",
                    type: "substrate",
                },
                {
                    name: "beefy",
                    account: "0x87D987206180B8f3807Dd90455606eEa85cdB87a",
                    type: "ethereum",
                },
                {
                    name: "parachain-primary-gov",
                    account: "0xeEBFA6B9242A19f91a0463291A937a20e3355681",
                    type: "ethereum",
                },
                {
                    name: "parachain-secondary-gov",
                    account: "0x13e16C4e5787f878f98a610EB321170512b134D4",
                    type: "ethereum",
                },
                {
                    name: "execution-assethub",
                    account: "5DF6KbMTBPGQN6ScjqXzdB2ngk5wi3wXvubpQVUZezNfM6aV",
                    type: "substrate",
                },
                {
                    name: "parachain-assethub",
                    account: "0x8b66D5499F52D6F1857084A61743dFCB9a712859",
                    type: "ethereum",
                },
                {
                    name: "execution-penpal",
                    account: "5HgmfVcc8xBUcReNJsUaJMhFWGkdYpEw4RiCX4SeZPdKXR6H",
                    type: "substrate",
                },
                {
                    name: "parachain-penpal",
                    account: "0x01F6749035e02205768f97e6f1d394Fb6769EC20",
                    type: "ethereum",
                },
            ],
            GRAPHQL_API_URL: "http://127.0.0.1/does/not/exist",
        },
    },
    paseo_sepolia: {
        name: "paseo_sepolia",
        ethChainId: 11155111,
        config: {
            BEACON_HTTP_API: "https://lodestar-sepolia.chainsafe.io",
            ETHEREUM_CHAINS: {
                "11155111": "https://ethereum-sepolia-rpc.publicnode.com",
            },
            RELAY_CHAIN_URL: "wss://paseo-rpc.dwellir.com",
            PARACHAINS: {
                "1000": "wss://asset-hub-paseo-rpc.dwellir.com",
                "1002": "wss://bridge-hub-paseo.dotters.network",
                "3369": "wss://paseo-muse-rpc.polkadot.io",
                "2043": `wss://parachain-testnet-rpc.origin-trail.network`,
            },
            GATEWAY_CONTRACT: "0x1607C1368bc943130258318c91bBd8cFf3D063E6",
            BEEFY_CONTRACT: "0x2c780945beb1241fE9c645800110cb9C4bBbb639",
            ASSET_HUB_PARAID: 1000,
            BRIDGE_HUB_PARAID: 1002,
            PRIMARY_GOVERNANCE_CHANNEL_ID:
                "0x0000000000000000000000000000000000000000000000000000000000000001",
            SECONDARY_GOVERNANCE_CHANNEL_ID:
                "0x0000000000000000000000000000000000000000000000000000000000000002",
            RELAYERS: [
                {
                    name: "beacon",
                    account: "5E4Hf7LzHE4W3jabjLWSP8p8RzEa9ednwRivFEwYAprzpgwc",
                    type: "substrate",
                },
                {
                    name: "beefy",
                    account: "0xc189De708158e75E5C88C0ABfA5F9a26C71F54D1",
                    type: "ethereum",
                },
                {
                    name: "parachain-primary-gov",
                    account: "0x4BBa8c0e87242897521Ba598d327bE8280032609",
                    type: "ethereum",
                },
                {
                    name: "parachain-secondary-gov",
                    account: "0x4BBa8c0e87242897521Ba598d327bE8280032609",
                    type: "ethereum",
                },
                {
                    name: "execution-assethub",
                    account: "5HT2ysqEg6SXghQ3NGXp1VWT22hhj48Um8UAwk6Udg8ZCEv8",
                    type: "substrate",
                },
                {
                    name: "parachain-assethub",
                    account: "0x4BBa8c0e87242897521Ba598d327bE8280032609",
                    type: "ethereum",
                },
            ],
            GRAPHQL_API_URL:
                "https://snowbridge.squids.live/snowbridge-subsquid-paseo@v1/api/graphql",
        },
    },
    polkadot_mainnet: {
        name: "polkadot_mainnet",
        ethChainId: 1,
        config: {
            BEACON_HTTP_API: "https://lodestar-mainnet.chainsafe.io",
            ETHEREUM_CHAINS: {
                "1": "https://ethereum-rpc.publicnode.com",
                "1284": "https://rpc.api.moonbeam.network",
            },
            RELAY_CHAIN_URL: "https://polkadot-rpc.n.dwellir.com",
            PARACHAINS: {
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
            GATEWAY_CONTRACT: "0x27ca963c279c93801941e1eb8799c23f407d68e7",
            BEEFY_CONTRACT: "0x1817874feAb3ce053d0F40AbC23870DB35C2AFfc",
            ASSET_HUB_PARAID: 1000,
            BRIDGE_HUB_PARAID: 1002,
            PRIMARY_GOVERNANCE_CHANNEL_ID:
                "0x0000000000000000000000000000000000000000000000000000000000000001",
            SECONDARY_GOVERNANCE_CHANNEL_ID:
                "0x0000000000000000000000000000000000000000000000000000000000000002",
            RELAYERS: [
                {
                    name: "beacon",
                    account: "16DWunYRv2q29SMxqgrPGhob5az332hhLggSj2Rysk3g1rvk",
                    type: "substrate",
                },
                {
                    name: "beefy",
                    account: "0xB8124B07467E46dE73eb5c73a7b1E03863F18062",
                    type: "ethereum",
                },
                {
                    name: "beefy-on-demand",
                    account: "0xF3D021D51a725F5DBDCE253248E826A8644Be3c1",
                    type: "ethereum",
                },
                {
                    name: "parachain-primary-gov",
                    account: "0x0f51678Ac675C1abf2BeC1DAC9cA701cFcfFF5E2",
                    type: "ethereum",
                },
                {
                    name: "parachain-secondary-gov",
                    account: "0x0f51678Ac675C1abf2BeC1DAC9cA701cFcfFF5E2",
                    type: "ethereum",
                },
                {
                    name: "execution-assethub",
                    account: "13Dbqvh6nLCRckyfsBr8wEJzxbi34KELwdYQFKKchN4NedGh",
                    type: "substrate",
                },
                {
                    name: "parachain-assethub",
                    account: "0x1F1819C3C68F9533adbB8E51C8E8428a818D693E",
                    type: "ethereum",
                },
            ],
            GRAPHQL_API_URL:
                "https://snowbridge.squids.live/snowbridge-subsquid-polkadot:production/api/graphql",
            TO_MONITOR_PARACHAINS: [2034, 2043, 3369], // Hydration, OriginTrail, Mythos
        },
        kusamaConfig: {
            ASSET_HUB_PARAID: 1000,
            BRIDGE_HUB_PARAID: 1002,
            PARACHAINS: {
                "1000": "wss://asset-hub-kusama-rpc.n.dwellir.com",
                "1002": "https://bridge-hub-kusama-rpc.n.dwellir.com",
            },
        },
    },
    westend_sepolia: {
        name: "westend_sepolia",
        ethChainId: 11155111,
        config: {
            BEACON_HTTP_API: "https://lodestar-sepolia.chainsafe.io",
            ETHEREUM_CHAINS: {
                "11155111": "https://ethereum-sepolia-rpc.publicnode.com",
            },
            RELAY_CHAIN_URL: "wss://westend-rpc.n.dwellir.com",
            PARACHAINS: {
                "1000": "wss://asset-hub-westend-rpc.n.dwellir.com",
                "1002": "wss://bridge-hub-westend-rpc.n.dwellir.com",
                "2313": `wss://node-7330371704012918784.nv.onfinality.io/ws?apikey=${
                    process.env["FREQUENCY_NODE_KEY"] ||
                    process.env["NEXT_PUBLIC_FREQUENCY_NODE_KEY"]
                }`,
            },
            GATEWAY_CONTRACT: "0x9ed8b47bc3417e3bd0507adc06e56e2fa360a4e9",
            BEEFY_CONTRACT: "0x6DFaD3D73A28c48E4F4c616ECda80885b415283a",
            ASSET_HUB_PARAID: 1000,
            BRIDGE_HUB_PARAID: 1002,
            PRIMARY_GOVERNANCE_CHANNEL_ID:
                "0x0000000000000000000000000000000000000000000000000000000000000001",
            SECONDARY_GOVERNANCE_CHANNEL_ID:
                "0x0000000000000000000000000000000000000000000000000000000000000002",
            RELAYERS: [
                {
                    name: "beacon",
                    account: "5E4Hf7LzHE4W3jabjLWSP8p8RzEa9ednwRivFEwYAprzpgwc",
                    type: "substrate",
                },
                {
                    name: "beefy",
                    account: "0x302f0b71b8ad3cf6dd90adb668e49b2168d652fd",
                    type: "ethereum",
                },
                {
                    name: "parachain-primary-gov",
                    account: "0x302f0b71b8ad3cf6dd90adb668e49b2168d652fd",
                    type: "ethereum",
                },
                {
                    name: "parachain-secondary-gov",
                    account: "0x302f0b71b8ad3cf6dd90adb668e49b2168d652fd",
                    type: "ethereum",
                },
                {
                    name: "execution-assethub",
                    account: "5E4Hf7LzHE4W3jabjLWSP8p8RzEa9ednwRivFEwYAprzpgwc",
                    type: "substrate",
                },
                {
                    name: "parachain-assethub",
                    account: "0x302f0b71b8ad3cf6dd90adb668e49b2168d652fd",
                    type: "ethereum",
                },
            ],
            GRAPHQL_API_URL:
                "https://snowbridge.squids.live/snowbridge-subsquid-westend@v1/api/graphql",
        },
    },
}
