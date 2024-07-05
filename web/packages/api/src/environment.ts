export type Config = {
    BEACON_HTTP_API: string
    ETHEREUM_API: (secret: string) => string
    RELAY_CHAIN_URL: string
    ASSET_HUB_URL: string
    BRIDGE_HUB_URL: string
    GATEWAY_CONTRACT: string
    BEEFY_CONTRACT: string
    ASSET_HUB_PARAID: number
    BRIDGE_HUB_PARAID: number
    PRIMARY_GOVERNANCE_CHANNEL_ID: string
    SECONDARY_GOVERNANCE_CHANNEL_ID: string
    RELAYERS: Relayer[]
    PARACHAINS: string[]
    SUBSCAN_API?: {
        RELAY_CHAIN_URL: string
        ASSET_HUB_URL: string
        BRIDGE_HUB_URL: string
    }
    GRAPHQL_API?: string
}

export type SourceType = "substrate" | "ethereum"
export type Relayer = { name: string; account: string; type: SourceType; balance?: bigint }
export type ParachainInfo = {
    paraId: number
    destinationFeeDOT: bigint
    skipExistentialDepositCheck: boolean
    has20ByteAccounts: boolean
    decimals: number
    maxConsumers: number
}
export type TransferToken = {
    id: string
    address: string
    minimumTransferAmount: bigint
}
export type TransferLocation = {
    id: string
    name: string
    type: SourceType
    destinationIds: string[]
    paraInfo?: ParachainInfo
    erc20tokensReceivable: TransferToken[]
}

export type SnowbridgeEnvironment = {
    config: Config
    name: string
    ethChainId: number
    locations: TransferLocation[]
}

export const SNOWBRIDGE_ENV: { [id: string]: SnowbridgeEnvironment } = {
    local_e2e: {
        name: "local_e2e",
        ethChainId: 11155111,
        locations: [
            {
                id: "ethereum",
                name: "Ethereum",
                type: "ethereum",
                destinationIds: ["assethub", "penpal"],
                erc20tokensReceivable: [
                    {
                        id: "WETH",
                        address: "0x87d1f7fdfEe7f651FaBc8bFCB6E086C278b77A7d",
                        minimumTransferAmount: 15_000_000_000_000n,
                    },
                ],
            },
            {
                id: "assethub",
                name: "Asset Hub",
                type: "substrate",
                destinationIds: ["ethereum"],
                paraInfo: {
                    paraId: 1000,
                    destinationFeeDOT: 0n,
                    skipExistentialDepositCheck: false,
                    has20ByteAccounts: false,
                    decimals: 12,
                    maxConsumers: 16,
                },
                erc20tokensReceivable: [
                    {
                        id: "WETH",
                        address: "0x87d1f7fdfEe7f651FaBc8bFCB6E086C278b77A7d",
                        minimumTransferAmount: 15_000_000_000_000n,
                    },
                ],
            },
            {
                id: "penpal",
                name: "Penpal",
                type: "substrate",
                destinationIds: ["ethereum"],
                paraInfo: {
                    paraId: 2000,
                    destinationFeeDOT: 4_000_000_000n,
                    skipExistentialDepositCheck: false,
                    has20ByteAccounts: false,
                    decimals: 12,
                    maxConsumers: 16,
                },
                erc20tokensReceivable: [
                    {
                        id: "WETH",
                        address: "0x87d1f7fdfEe7f651FaBc8bFCB6E086C278b77A7d",
                        minimumTransferAmount: 1n,
                    },
                ],
            },
        ],
        config: {
            BEACON_HTTP_API: "http://127.0.0.1:9596",
            ETHEREUM_API: () => "ws://127.0.0.1:8546",
            RELAY_CHAIN_URL: "ws://127.0.0.1:9944",
            ASSET_HUB_URL: "ws://127.0.0.1:12144",
            BRIDGE_HUB_URL: "ws://127.0.0.1:11144",
            PARACHAINS: ["ws://127.0.0.1:13144"],
            GATEWAY_CONTRACT: "0xEDa338E4dC46038493b885327842fD3E301CaB39",
            BEEFY_CONTRACT: "0x992B9df075935E522EC7950F37eC8557e86f6fdb",
            ASSET_HUB_PARAID: 1000,
            BRIDGE_HUB_PARAID: 1013,
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
        },
    },
    rococo_sepolia: {
        name: "rococo_sepolia",
        ethChainId: 11155111,
        locations: [
            {
                id: "ethereum",
                name: "Ethereum",
                type: "ethereum",
                destinationIds: ["assethub", "muse"],
                erc20tokensReceivable: [
                    {
                        id: "WETH",
                        address: "0xfff9976782d46cc05630d1f6ebab18b2324d6b14",
                        minimumTransferAmount: 15_000_000_000_000n,
                    },
                    {
                        id: "vETH",
                        address: "0xc3d088842dcf02c13699f936bb83dfbbc6f721ab",
                        minimumTransferAmount: 1n,
                    },
                    {
                        id: "MUSE",
                        address: "0xb34a6924a02100ba6ef12af1c798285e8f7a16ee",
                        minimumTransferAmount: 1n,
                    },
                ],
            },
            {
                id: "assethub",
                name: "Asset Hub",
                type: "substrate",
                destinationIds: ["ethereum"],
                paraInfo: {
                    paraId: 1000,
                    destinationFeeDOT: 0n,
                    skipExistentialDepositCheck: false,
                    has20ByteAccounts: false,
                    decimals: 12,
                    maxConsumers: 16,
                },
                erc20tokensReceivable: [
                    {
                        id: "WETH",
                        address: "0xfff9976782d46cc05630d1f6ebab18b2324d6b14",
                        minimumTransferAmount: 15_000_000_000_000n,
                    },
                    {
                        id: "vETH",
                        address: "0xc3d088842dcf02c13699f936bb83dfbbc6f721ab",
                        minimumTransferAmount: 1n,
                    },
                    {
                        id: "MUSE",
                        address: "0xb34a6924a02100ba6ef12af1c798285e8f7a16ee",
                        minimumTransferAmount: 1n,
                    },
                ],
            },
            {
                id: "muse",
                name: "Muse",
                type: "substrate",
                destinationIds: [],
                paraInfo: {
                    paraId: 3369,
                    destinationFeeDOT: 200_000_000_000n,
                    skipExistentialDepositCheck: true,
                    has20ByteAccounts: true,
                    decimals: 18,
                    maxConsumers: 16,
                },
                erc20tokensReceivable: [
                    {
                        id: "MUSE",
                        address: "0xb34a6924a02100ba6ef12af1c798285e8f7a16ee",
                        minimumTransferAmount: 10_000_000_000_000_000n,
                    },
                ],
            },
        ],
        config: {
            BEACON_HTTP_API: "https://lodestar-sepolia.chainsafe.io",
            ETHEREUM_API: (key) => `https://eth-sepolia.g.alchemy.com/v2/${key}`,
            RELAY_CHAIN_URL: "https://rococo-rpc.polkadot.io",
            ASSET_HUB_URL: "wss://rococo-asset-hub-rpc.polkadot.io",
            BRIDGE_HUB_URL: "https://rococo-bridge-hub-rpc.polkadot.io",
            PARACHAINS: ["https://rococo-muse-rpc.polkadot.io"],
            GATEWAY_CONTRACT: "0x5b4909ce6ca82d2ce23bd46738953c7959e710cd",
            BEEFY_CONTRACT: "0x27e5e17ac995d3d720c311e1e9560e28f5855fb1",
            ASSET_HUB_PARAID: 1000,
            BRIDGE_HUB_PARAID: 1013,
            PRIMARY_GOVERNANCE_CHANNEL_ID:
                "0x0000000000000000000000000000000000000000000000000000000000000001",
            SECONDARY_GOVERNANCE_CHANNEL_ID:
                "0x0000000000000000000000000000000000000000000000000000000000000002",
            RELAYERS: [
                {
                    name: "beacon",
                    account: "5FyC9GkHhiAYjMtddwVNc2gx8wBjH9gpMKWbQ1QVXmmJtr8M",
                    type: "substrate",
                },
                {
                    name: "beefy",
                    account: "0xF061685F2B729b89a7A5966B3ab9aee15269e8FE",
                    type: "ethereum",
                },
                {
                    name: "parachain-primary-gov",
                    account: "0xE3f4e40E0dB2F828e248dB2790958035BaB1a4B5",
                    type: "ethereum",
                },
                {
                    name: "parachain-secondary-gov",
                    account: "0x71429d3B9d68557C2F49e42e12B9cf7edAADCe3D",
                    type: "ethereum",
                },
                {
                    name: "execution-assethub",
                    account: "5GeLu5rPcaoZ2RVDbhX8DKJt8NxnKn6DvvjfuhnwTZyYRY59",
                    type: "substrate",
                },
                {
                    name: "parachain-assethub",
                    account: "0x0b65d43d159f1C40Bad7768fd59667E3104a2ECE",
                    type: "ethereum",
                },
            ],
            SUBSCAN_API: {
                RELAY_CHAIN_URL: "https://rococo.api.subscan.io",
                ASSET_HUB_URL: "https://assethub-rococo.api.subscan.io",
                BRIDGE_HUB_URL: "https://bridgehub-rococo.api.subscan.io",
            },
        },
    },
    polkadot_mainnet: {
        name: "polkadot_mainnet",
        ethChainId: 1,
        locations: [
            {
                id: "ethereum",
                name: "Ethereum",
                type: "ethereum",
                destinationIds: ["assethub", "mythos"],
                erc20tokensReceivable: [
                    {
                        id: "WETH",
                        address: "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2",
                        minimumTransferAmount: 15_000_000_000_000n,
                    },
                    {
                        id: "WBTC",
                        address: "0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599",
                        minimumTransferAmount: 1n,
                    },
                    {
                        id: "SHIB",
                        address: "0x95aD61b0a150d79219dCF64E1E6Cc01f0B64C4cE",
                        minimumTransferAmount: 1n,
                    },
                    {
                        id: "PEPE",
                        address: "0x6982508145454Ce325dDbE47a25d4ec3d2311933",
                        minimumTransferAmount: 1n,
                    },
                    {
                        id: "TON",
                        address: "0x582d872A1B094FC48F5DE31D3B73F2D9bE47def1",
                        minimumTransferAmount: 1n,
                    },
                    {
                        id: "wstETH",
                        address: "0x7f39C581F595B53c5cb19bD0b3f8dA6c935E2Ca0",
                        minimumTransferAmount: 1n,
                    },
                    {
                        id: "tBTC",
                        address: "0x18084fbA666a33d37592fA2633fD49a74DD93a88",
                        minimumTransferAmount: 1n,
                    },
                ],
            },
            {
                id: "assethub",
                name: "Asset Hub",
                type: "substrate",
                destinationIds: ["ethereum"],
                paraInfo: {
                    paraId: 1000,
                    destinationFeeDOT: 0n,
                    skipExistentialDepositCheck: false,
                    has20ByteAccounts: false,
                    decimals: 10,
                    maxConsumers: 64,
                },
                erc20tokensReceivable: [
                    {
                        id: "WETH",
                        address: "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2",
                        minimumTransferAmount: 15_000_000_000_000n,
                    },
                    {
                        id: "WBTC",
                        address: "0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599",
                        minimumTransferAmount: 1n,
                    },
                    {
                        id: "SHIB",
                        address: "0x95aD61b0a150d79219dCF64E1E6Cc01f0B64C4cE",
                        minimumTransferAmount: 1n,
                    },
                    {
                        id: "PEPE",
                        address: "0x6982508145454Ce325dDbE47a25d4ec3d2311933",
                        minimumTransferAmount: 1n,
                    },
                    {
                        id: "TON",
                        address: "0x582d872A1B094FC48F5DE31D3B73F2D9bE47def1",
                        minimumTransferAmount: 1n,
                    },
                    {
                        id: "wstETH",
                        address: "0x7f39C581F595B53c5cb19bD0b3f8dA6c935E2Ca0",
                        minimumTransferAmount: 1n,
                    },
                    {
                        id: "tBTC",
                        address: "0x18084fbA666a33d37592fA2633fD49a74DD93a88",
                        minimumTransferAmount: 1n,
                    },
                ],
            },
            {
                id: "mythos",
                name: "Mythos",
                type: "substrate",
                destinationIds: [],
                paraInfo: {
                    paraId: 3369,
                    destinationFeeDOT: 500_000_000n,
                    skipExistentialDepositCheck: true,
                    has20ByteAccounts: true,
                    decimals: 18,
                    maxConsumers: 16,
                },
                erc20tokensReceivable: [
                    {
                        id: "MYTH",
                        address: "0xba41ddf06b7ffd89d1267b5a93bfef2424eb2003",
                        minimumTransferAmount: 10_000_000_000_000_000n,
                    },
                ],
            },
        ],
        config: {
            BEACON_HTTP_API: "https://lodestar-mainnet.chainsafe.io",
            ETHEREUM_API: (key) => `https://eth-mainnet.g.alchemy.com/v2/${key}`,
            RELAY_CHAIN_URL: "https://polkadot-rpc.dwellir.com",
            ASSET_HUB_URL: "wss://asset-hub-polkadot-rpc.dwellir.com",
            BRIDGE_HUB_URL: "https://bridge-hub-polkadot-rpc.dwellir.com",
            PARACHAINS: ["https://polkadot-mythos-rpc.polkadot.io"],
            GATEWAY_CONTRACT: "0x27ca963c279c93801941e1eb8799c23f407d68e7",
            BEEFY_CONTRACT: "0x6eD05bAa904df3DE117EcFa638d4CB84e1B8A00C",
            ASSET_HUB_PARAID: 1000,
            BRIDGE_HUB_PARAID: 1002,
            PRIMARY_GOVERNANCE_CHANNEL_ID:
                "0x0000000000000000000000000000000000000000000000000000000000000001",
            SECONDARY_GOVERNANCE_CHANNEL_ID:
                "0x0000000000000000000000000000000000000000000000000000000000000002",
            RELAYERS: [
                {
                    name: "beacon",
                    account: "5HHDmTHN4FZYhuMSt3oP8YySDxzPLj9ZGBwxZjSdKf29qcnj",
                    type: "substrate",
                },
                {
                    name: "beefy",
                    account: "0xB8124B07467E46dE73eb5c73a7b1E03863F18062",
                    type: "ethereum",
                },
                {
                    name: "parachain-primary-gov",
                    account: "0x1F1819C3C68F9533adbB8E51C8E8428a818D693E",
                    type: "ethereum",
                },
                {
                    name: "parachain-secondary-gov",
                    account: "0x1F1819C3C68F9533adbB8E51C8E8428a818D693E",
                    type: "ethereum",
                },
                {
                    name: "execution-assethub",
                    account: "5HHDmTHN4FZYhuMSt3oP8YySDxzPLj9ZGBwxZjSdKf29qcnj",
                    type: "substrate",
                },
                {
                    name: "parachain-assethub",
                    account: "0x1F1819C3C68F9533adbB8E51C8E8428a818D693E",
                    type: "ethereum",
                },
            ],
            SUBSCAN_API: {
                RELAY_CHAIN_URL: "https://polkadot.api.subscan.io",
                ASSET_HUB_URL: "https://assethub-polkadot.api.subscan.io",
                BRIDGE_HUB_URL: "https://bridgehub-polkadot.api.subscan.io",
            },
            // todo: For mainnet customize dns service with A record ip point to https://data.snowbridge.network/graphql
            GRAPHQL_API: "https://rococo-rpc.snowbridge.network/graphql",
        },
    },
}
