export type MonitorRelayer = {
    name: string
    account: string
    type: "substrate" | "ethereum"
    balance?: bigint
}

export type MonitorEnvironmentConfig = {
    PRIMARY_GOVERNANCE_CHANNEL_ID: string
    SECONDARY_GOVERNANCE_CHANNEL_ID: string
    RELAYERS: MonitorRelayer[]
    TO_MONITOR_CHAINS?: { id: string; name?: string; type: "substrate" | "ethereum" }[]
}

export const monitorParams: {
    [id: string]: MonitorEnvironmentConfig
} = {
    local_e2e: {
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
    paseo_sepolia: {
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
    },
    polkadot_mainnet: {
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
                name: "ethereum-assethub",
                account: "13Dbqvh6nLCRckyfsBr8wEJzxbi34KELwdYQFKKchN4NedGh",
                type: "substrate",
            },
            {
                name: "ethereum-assethub-v2",
                account: "131ESXiBY3kaNzwWrKh6mFZE7QqxQsR8Yj7rJLCxMF4okBef",
                type: "substrate",
            },
            {
                name: "parachain-assethub-v3",
                account: "0xBa9bC9a8Aa87872f7B990031bde984A00b9CEd49",
                type: "ethereum",
            },
        ],
        TO_MONITOR_CHAINS: [
            { id: "1000", name: "AssetHub", type: "substrate" },
            { id: "kusama_1000", name: "AssetHub on Westend", type: "substrate" },
            { id: "1002", name: "BridgeHub", type: "substrate" },
            { id: "2004", name: "Moonbeam", type: "substrate" },
            { id: "2034", name: "Hydration", type: "substrate" },
            { id: "2043", name: "Neuroweb", type: "substrate" },
            { id: "3369", name: "Mythos", type: "substrate" },
            { id: "2030", name: "Bifrost", type: "substrate" },
            { id: "2000", name: "Acala", type: "substrate" },
            { id: "1", name: "Ethereum Mainnet", type: "ethereum" },
            { id: "10", name: "Optimism", type: "ethereum" },
            { id: "8453", name: "Base", type: "ethereum" },
            { id: "42161", name: "Arbitrum", type: "ethereum" },
        ],
    },
    westend_sepolia: {
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
    },
}
