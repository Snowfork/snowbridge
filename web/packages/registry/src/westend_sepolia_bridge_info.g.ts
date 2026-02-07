const registry = {
    environment: {
        name: "westend_sepolia",
        ethChainId: 11155111,
        beaconApiUrl: "https://lodestar-sepolia.chainsafe.io",
        ethereumChains: {
            "84532": "https://base-sepolia-rpc.publicnode.com",
            "11155111": "https://ethereum-sepolia-rpc.publicnode.com",
        },
        relaychainUrl: "wss://westend-rpc.n.dwellir.com",
        parachains: {
            "1000": "wss://asset-hub-westend-rpc.n.dwellir.com",
            "1002": "wss://bridge-hub-westend-rpc.n.dwellir.com",
        },
        gatewayContract: "0x9ed8b47bc3417e3bd0507adc06e56e2fa360a4e9",
        beefyContract: "0xa04460b1d8bbef33f54edb2c3115e3e4d41237a6",
        assetHubParaId: 1000,
        bridgeHubParaId: 1002,
        v2_parachains: [1000],
        indexerGraphQlUrl:
            "https://snowbridge.squids.live/snowbridge-subsquid-westend@v1/api/graphql",
        l2Bridge: {
            acrossAPIUrl: "https://testnet.across.to/api",
            l1AdapterAddress: "0xa5b8589bd534701be49916c4d2e634ab1c765cbf",
            l1HandlerAddress: "0x924a9f036260ddd5808007e1aa95f08ed08aa569",
            l1FeeTokenAddress: "0xfff9976782d46cc05630d1f6ebab18b2324d6b14",
            l1SwapRouterAddress: "0x3bfa4769fb09eefc5a80d6e87c3b9c650f7ae48e",
            l1SwapQuoterAddress: "0xed1f6473345f45b75f8179591dd5ba1888cf2fb3",
            l2Chains: {
                "84532": {
                    adapterAddress: "0xf06939613a3838af11104c898758220db9093679",
                    feeTokenAddress: "0x4200000000000000000000000000000000000006",
                    swapRoutes: [
                        {
                            inputToken: "0x4200000000000000000000000000000000000006",
                            outputToken: "0xfff9976782d46cc05630d1f6ebab18b2324d6b14",
                            swapFee: 0,
                        },
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
    routes: [
        {
            from: {
                kind: "ethereum",
                id: 11155111,
            },
            to: {
                kind: "polkadot",
                id: 1000,
            },
            assets: [
                "0x0000000000000000000000000000000000000000",
                "0x1c7d4b196cb0c7b01d743fbc6116a902379c7238",
                "0xfff9976782d46cc05630d1f6ebab18b2324d6b14",
                "0x23838b1bb57cecf4422a57dd8e7f8a087b30d54f",
                "0xb8a0f2703ac6bdd352096c90c2945a097e8f4055",
                "0xf50fb50d65c8c1f6c72e4d8397c984933afc8f7e",
            ],
        },
        {
            from: {
                kind: "polkadot",
                id: 1000,
            },
            to: {
                kind: "ethereum",
                id: 11155111,
            },
            assets: [
                "0x0000000000000000000000000000000000000000",
                "0x1c7d4b196cb0c7b01d743fbc6116a902379c7238",
                "0xfff9976782d46cc05630d1f6ebab18b2324d6b14",
                "0x23838b1bb57cecf4422a57dd8e7f8a087b30d54f",
                "0xb8a0f2703ac6bdd352096c90c2945a097e8f4055",
                "0xf50fb50d65c8c1f6c72e4d8397c984933afc8f7e",
            ],
        },
        {
            from: {
                kind: "polkadot",
                id: 1000,
            },
            to: {
                kind: "ethereum_l2",
                id: 84532,
            },
            assets: [
                "0x0000000000000000000000000000000000000000",
                "0x1c7d4b196cb0c7b01d743fbc6116a902379c7238",
                "0xfff9976782d46cc05630d1f6ebab18b2324d6b14",
            ],
        },
        {
            from: {
                kind: "ethereum_l2",
                id: 84532,
            },
            to: {
                kind: "polkadot",
                id: 1000,
            },
            assets: [
                "0x1c7d4b196cb0c7b01d743fbc6116a902379c7238",
                "0xfff9976782d46cc05630d1f6ebab18b2324d6b14",
            ],
        },
    ],
    registry: {
        timestamp: "2026-02-07T00:46:49.479Z",
        environment: "westend_sepolia",
        ethChainId: 11155111,
        gatewayAddress: "0x9ed8b47bc3417e3bd0507adc06e56e2fa360a4e9",
        assetHubParaId: 1000,
        bridgeHubParaId: 1002,
        relaychain: {
            tokenSymbols: "WND",
            tokenDecimals: 12,
            ss58Format: 42,
            isEthereum: false,
            accountType: "AccountId32",
            name: "Westend",
            specName: "westend",
            specVersion: 1021001,
        },
        bridgeHub: {
            tokenSymbols: "WND",
            tokenDecimals: 12,
            ss58Format: 42,
            isEthereum: false,
            accountType: "AccountId32",
            name: "Westend BridgeHub",
            specName: "bridge-hub-westend",
            specVersion: 1021001,
        },
        ethereumChains: {
            ethereum_l2_84532: {
                kind: "ethereum_l2",
                id: 84532,
                name: "base-sepolia",
                assets: {
                    "0x4200000000000000000000000000000000000006": {
                        token: "0x4200000000000000000000000000000000000006",
                        name: "Wrapped Ether",
                        symbol: "WETH",
                        decimals: 18,
                        swapTokenAddress: "0xfff9976782d46cc05630d1f6ebab18b2324d6b14",
                        swapFee: 0,
                    },
                    "0x036cbd53842c5426634e7929541ec2318f3dcf7e": {
                        token: "0x036cbd53842c5426634e7929541ec2318f3dcf7e",
                        name: "USDC",
                        symbol: "USDC",
                        decimals: 6,
                        swapTokenAddress: "0x1c7d4b196cb0c7b01d743fbc6116a902379c7238",
                        swapFee: 500,
                    },
                    "0x0000000000000000000000000000000000000000": {
                        token: "0x0000000000000000000000000000000000000000",
                        name: "Ether",
                        symbol: "Ether",
                        decimals: 18,
                        swapTokenAddress: "0x0000000000000000000000000000000000000000",
                        swapFee: 0,
                    },
                },
                key: "ethereum_l2_84532",
            },
            ethereum_11155111: {
                kind: "ethereum",
                id: 11155111,
                name: "sepolia",
                assets: {
                    "0x0000000000000000000000000000000000000000": {
                        token: "0x0000000000000000000000000000000000000000",
                        name: "Ether",
                        symbol: "Ether",
                        decimals: 18,
                    },
                    "0x1c7d4b196cb0c7b01d743fbc6116a902379c7238": {
                        token: "0x1c7d4b196cb0c7b01d743fbc6116a902379c7238",
                        name: "USDC",
                        symbol: "USDC",
                        decimals: 6,
                        deliveryGas: 80000n,
                    },
                    "0x72c610e05eaafcdf1fa7a2da15374ee90edb1620": {
                        token: "0x72c610e05eaafcdf1fa7a2da15374ee90edb1620",
                        name: "Frequency",
                        symbol: "eFRQCY",
                        decimals: 12,
                        deliveryGas: 80000n,
                    },
                    "0xfff9976782d46cc05630d1f6ebab18b2324d6b14": {
                        token: "0xfff9976782d46cc05630d1f6ebab18b2324d6b14",
                        name: "Wrapped Ether",
                        symbol: "WETH",
                        decimals: 18,
                        deliveryGas: 80000n,
                    },
                    "0x23838b1bb57cecf4422a57dd8e7f8a087b30d54f": {
                        token: "0x23838b1bb57cecf4422a57dd8e7f8a087b30d54f",
                        name: "Frequency",
                        symbol: "XRQCY",
                        decimals: 8,
                        foreignId:
                            "0xaf13384cf9612ef1ff4b87470ab247d6f8d8110d4f5af2fe290ce6767818712c",
                        deliveryGas: 80000n,
                    },
                    "0xb8a0f2703ac6bdd352096c90c2945a097e8f4055": {
                        token: "0xb8a0f2703ac6bdd352096c90c2945a097e8f4055",
                        name: "WND",
                        symbol: "WND",
                        decimals: 12,
                        foreignId:
                            "0x2121cfe35065c0c33465fbada265f08e9613428a4b9eb4bb717cd7db2abf622e",
                        deliveryGas: 80000n,
                    },
                    "0xf50fb50d65c8c1f6c72e4d8397c984933afc8f7e": {
                        token: "0xf50fb50d65c8c1f6c72e4d8397c984933afc8f7e",
                        name: "WND",
                        symbol: "WND",
                        decimals: 12,
                        foreignId:
                            "0x9441dceeeffa7e032eedaccf9b7632e60e86711551a82ffbbb0dda8afd9e4ef7",
                        deliveryGas: 80000n,
                    },
                },
                key: "ethereum_11155111",
                baseDeliveryGas: 120000n,
            },
        },
        parachains: {
            polkadot_1000: {
                id: 1000,
                kind: "polkadot",
                key: "polkadot_1000",
                features: {
                    hasPalletXcm: true,
                    hasDryRunApi: true,
                    hasTxPaymentApi: true,
                    hasDryRunRpc: true,
                    hasDotBalance: true,
                    hasEthBalance: true,
                    hasXcmPaymentApi: true,
                    supportsAliasOrigin: true,
                    xcmVersion: "v5",
                    supportsV2: true,
                },
                info: {
                    tokenSymbols: "WND",
                    tokenDecimals: 12,
                    ss58Format: 42,
                    isEthereum: false,
                    accountType: "AccountId32",
                    name: "Westend Asset Hub",
                    specName: "westmint",
                    specVersion: 1021002,
                },
                assets: {
                    "0x0000000000000000000000000000000000000000": {
                        token: "0x0000000000000000000000000000000000000000",
                        name: "Ether",
                        minimumBalance: 15000n,
                        symbol: "Ether",
                        decimals: 18,
                        isSufficient: true,
                    },
                    "0x1c7d4b196cb0c7b01d743fbc6116a902379c7238": {
                        token: "0x1c7d4b196cb0c7b01d743fbc6116a902379c7238",
                        name: "",
                        minimumBalance: 1n,
                        symbol: "",
                        decimals: 0,
                        isSufficient: false,
                    },
                    "0x72c610e05eaafcdf1fa7a2da15374ee90edb1620": {
                        token: "0x72c610e05eaafcdf1fa7a2da15374ee90edb1620",
                        name: "",
                        minimumBalance: 1n,
                        symbol: "",
                        decimals: 0,
                        isSufficient: false,
                    },
                    "0xfff9976782d46cc05630d1f6ebab18b2324d6b14": {
                        token: "0xfff9976782d46cc05630d1f6ebab18b2324d6b14",
                        name: "Wrapped Ether",
                        minimumBalance: 15000000000000n,
                        symbol: "WETH",
                        decimals: 18,
                        isSufficient: true,
                    },
                    "0x23838b1bb57cecf4422a57dd8e7f8a087b30d54f": {
                        token: "0x23838b1bb57cecf4422a57dd8e7f8a087b30d54f",
                        name: "",
                        symbol: "",
                        decimals: 0,
                        locationOnEthereum: {
                            parents: 1,
                            interior: {
                                x2: [
                                    {
                                        globalConsensus: {
                                            byGenesis:
                                                "0xe143f23803ac50e8f6f8e62695d1ce9e4e1d68aa36c1cd2cfd15340213f3423e",
                                        },
                                    },
                                    {
                                        parachain: 2313,
                                    },
                                ],
                            },
                        },
                        location: {
                            parents: 1,
                            interior: {
                                x1: [
                                    {
                                        parachain: 2313,
                                    },
                                ],
                            },
                        },
                        locationOnAH: {
                            parents: 1,
                            interior: {
                                x1: [
                                    {
                                        parachain: 2313,
                                    },
                                ],
                            },
                        },
                        foreignId:
                            "0xaf13384cf9612ef1ff4b87470ab247d6f8d8110d4f5af2fe290ce6767818712c",
                        minimumBalance: 1n,
                        isSufficient: false,
                    },
                    "0xb8a0f2703ac6bdd352096c90c2945a097e8f4055": {
                        token: "0xb8a0f2703ac6bdd352096c90c2945a097e8f4055",
                        name: "",
                        symbol: "WND",
                        decimals: 12,
                        locationOnEthereum: {
                            parents: 1,
                            interior: {
                                x1: [
                                    {
                                        globalConsensus: {
                                            byGenesis:
                                                "0xe143f23803ac50e8f6f8e62695d1ce9e4e1d68aa36c1cd2cfd15340213f3423e",
                                        },
                                    },
                                ],
                            },
                        },
                        location: {
                            parents: 1,
                            interior: "Here",
                        },
                        locationOnAH: {
                            parents: 1,
                            interior: "Here",
                        },
                        foreignId:
                            "0x2121cfe35065c0c33465fbada265f08e9613428a4b9eb4bb717cd7db2abf622e",
                        minimumBalance: 1000000000n,
                        isSufficient: true,
                    },
                    "0xf50fb50d65c8c1f6c72e4d8397c984933afc8f7e": {
                        token: "0xf50fb50d65c8c1f6c72e4d8397c984933afc8f7e",
                        name: "",
                        symbol: "WND",
                        decimals: 12,
                        locationOnEthereum: {
                            parents: 1,
                            interior: {
                                x1: [
                                    {
                                        globalConsensus: {
                                            byGenesis:
                                                "0xe143f23803ac50e8f6f8e62695d1ce9e4e1d68aa36c1cd2cfd15340213f3423e",
                                        },
                                    },
                                ],
                            },
                        },
                        location: {
                            parents: 1,
                            interior: "Here",
                        },
                        locationOnAH: {
                            parents: 1,
                            interior: "Here",
                        },
                        foreignId:
                            "0x9441dceeeffa7e032eedaccf9b7632e60e86711551a82ffbbb0dda8afd9e4ef7",
                        minimumBalance: 1000000000n,
                        isSufficient: true,
                    },
                },
                estimatedExecutionFeeDOT: 0n,
                estimatedDeliveryFeeDOT: 0n,
            },
        },
    },
} as const
export default registry
