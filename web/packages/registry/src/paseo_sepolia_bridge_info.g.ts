const registry = {
    "environment": {
        "name": "paseo_sepolia",
        "ethChainId": 11155111,
        "beaconApiUrl": "https://lodestar-sepolia.chainsafe.io",
        "ethereumChains": {
            "11155111": "https://ethereum-sepolia-rpc.publicnode.com"
        },
        "relaychainUrl": "wss://paseo-rpc.n.dwellir.com",
        "parachains": {
            "1000": "wss://asset-hub-paseo-rpc.n.dwellir.com",
            "1002": "wss://bridge-hub-paseo.dotters.network",
            "2043": "wss://parachain-testnet-rpc.origin-trail.network",
            "3369": "wss://paseo-muse-rpc.polkadot.io"
        },
        "gatewayContract": "0x1607C1368bc943130258318c91bBd8cFf3D063E6",
        "beefyContract": "0x2c780945beb1241fE9c645800110cb9C4bBbb639",
        "assetHubParaId": 1000,
        "bridgeHubParaId": 1002,
        "v2_parachains": [
            1000
        ],
        "indexerGraphQlUrl": "https://snowbridge.squids.live/snowbridge-subsquid-paseo@v1/api/graphql",
        "metadataOverrides": {
            "0xef32abea56beff54f61da319a7311098d6fbcea9": {
                "name": "OriginTrail TRAC",
                "symbol": "TRAC"
            }
        }
    },
    "routes": [
        {
            "type": "ethereum",
            "id": "ethereum",
            "key": "11155111",
            "destinations": {
                "1000": {
                    "type": "substrate",
                    "assets": [
                        "0x0000000000000000000000000000000000000000",
                        "0x22e12ed4e6bcde652a73552dde340fcb972eef89",
                        "0xef32abea56beff54f61da319a7311098d6fbcea9",
                        "0x99e743964c036bc28931fb564817db428aa7f752",
                        "0xfff9976782d46cc05630d1f6ebab18b2324d6b14"
                    ]
                },
                "2043": {
                    "type": "substrate",
                    "assets": [
                        "0xef32abea56beff54f61da319a7311098d6fbcea9"
                    ]
                },
                "3369": {
                    "type": "substrate",
                    "assets": [
                        "0xb34a6924a02100ba6ef12af1c798285e8f7a16ee"
                    ]
                }
            }
        },
        {
            "type": "substrate",
            "id": "asset-hub-paseo",
            "key": "1000",
            "destinations": {
                "11155111": {
                    "type": "ethereum",
                    "assets": [
                        "0x0000000000000000000000000000000000000000",
                        "0x22e12ed4e6bcde652a73552dde340fcb972eef89",
                        "0xef32abea56beff54f61da319a7311098d6fbcea9",
                        "0x99e743964c036bc28931fb564817db428aa7f752",
                        "0xfff9976782d46cc05630d1f6ebab18b2324d6b14"
                    ]
                }
            }
        },
        {
            "type": "substrate",
            "id": "origintrail-parachain",
            "key": "2043",
            "destinations": {
                "11155111": {
                    "type": "ethereum",
                    "assets": [
                        "0xef32abea56beff54f61da319a7311098d6fbcea9"
                    ]
                }
            }
        },
        {
            "type": "substrate",
            "id": "muse",
            "key": "3369",
            "destinations": {
                "11155111": {
                    "type": "ethereum",
                    "assets": [
                        "0xb34a6924a02100ba6ef12af1c798285e8f7a16ee"
                    ]
                }
            }
        }
    ],
    "registry": {
        "timestamp": "2026-01-26T23:04:32.597Z",
        "environment": "paseo_sepolia",
        "ethChainId": 11155111,
        "gatewayAddress": "0x1607C1368bc943130258318c91bBd8cFf3D063E6",
        "assetHubParaId": 1000,
        "bridgeHubParaId": 1002,
        "relaychain": {
            "tokenSymbols": "PAS",
            "tokenDecimals": 10,
            "ss58Format": 0,
            "isEthereum": false,
            "accountType": "AccountId32",
            "name": "Paseo Testnet",
            "specName": "paseo",
            "specVersion": 2000004
        },
        "bridgeHub": {
            "tokenSymbols": "PAS",
            "tokenDecimals": 10,
            "ss58Format": 0,
            "isEthereum": false,
            "accountType": "AccountId32",
            "name": "Paseo Bridge Hub",
            "specName": "bridge-hub-paseo",
            "specVersion": 2000004
        },
        "ethereumChains": {
            "11155111": {
                "chainId": 11155111,
                "assets": {
                    "0x0000000000000000000000000000000000000000": {
                        "token": "0x0000000000000000000000000000000000000000",
                        "name": "Ether",
                        "symbol": "ETH",
                        "decimals": 18
                    },
                    "0xb34a6924a02100ba6ef12af1c798285e8f7a16ee": {
                        "token": "0xb34a6924a02100ba6ef12af1c798285e8f7a16ee",
                        "name": "Muse",
                        "symbol": "MUSE",
                        "decimals": 18,
                        "deliveryGas": 80000n
                    },
                    "0x22e12ed4e6bcde652a73552dde340fcb972eef89": {
                        "token": "0x22e12ed4e6bcde652a73552dde340fcb972eef89",
                        "name": "Wrapped PILT",
                        "symbol": "wPILT",
                        "decimals": 15,
                        "deliveryGas": 80000n
                    },
                    "0xef32abea56beff54f61da319a7311098d6fbcea9": {
                        "token": "0xef32abea56beff54f61da319a7311098d6fbcea9",
                        "name": "OriginTrail TRAC",
                        "symbol": "TRAC",
                        "decimals": 18,
                        "deliveryGas": 80000n
                    },
                    "0x99e743964c036bc28931fb564817db428aa7f752": {
                        "token": "0x99e743964c036bc28931fb564817db428aa7f752",
                        "name": "KILT",
                        "symbol": "KILT",
                        "decimals": 15,
                        "deliveryGas": 80000n
                    },
                    "0xfff9976782d46cc05630d1f6ebab18b2324d6b14": {
                        "token": "0xfff9976782d46cc05630d1f6ebab18b2324d6b14",
                        "name": "Wrapped Ether",
                        "symbol": "WETH",
                        "decimals": 18,
                        "deliveryGas": 80000n
                    }
                },
                "id": "sepolia",
                "baseDeliveryGas": 120000n
            }
        },
        "parachains": {
            "1000": {
                "parachainId": 1000,
                "features": {
                    "hasPalletXcm": true,
                    "hasDryRunApi": true,
                    "hasTxPaymentApi": true,
                    "hasDryRunRpc": true,
                    "hasDotBalance": true,
                    "hasEthBalance": true,
                    "hasXcmPaymentApi": true,
                    "supportsAliasOrigin": true,
                    "xcmVersion": "v5",
                    "supportsV2": true
                },
                "info": {
                    "tokenSymbols": "PAS",
                    "tokenDecimals": 10,
                    "ss58Format": 0,
                    "isEthereum": false,
                    "accountType": "AccountId32",
                    "name": "Paseo Asset Hub",
                    "specName": "asset-hub-paseo",
                    "specVersion": 2000004
                },
                "assets": {
                    "0x0000000000000000000000000000000000000000": {
                        "token": "0x0000000000000000000000000000000000000000",
                        "name": "Ether",
                        "minimumBalance": 15000000000000n,
                        "symbol": "ETH",
                        "decimals": 18,
                        "isSufficient": true
                    },
                    "0xb34a6924a02100ba6ef12af1c798285e8f7a16ee": {
                        "token": "0xb34a6924a02100ba6ef12af1c798285e8f7a16ee",
                        "name": "",
                        "minimumBalance": 1n,
                        "symbol": "",
                        "decimals": 0,
                        "isSufficient": false
                    },
                    "0x22e12ed4e6bcde652a73552dde340fcb972eef89": {
                        "token": "0x22e12ed4e6bcde652a73552dde340fcb972eef89",
                        "name": "",
                        "minimumBalance": 1n,
                        "symbol": "",
                        "decimals": 0,
                        "isSufficient": false
                    },
                    "0xef32abea56beff54f61da319a7311098d6fbcea9": {
                        "token": "0xef32abea56beff54f61da319a7311098d6fbcea9",
                        "name": "",
                        "minimumBalance": 1n,
                        "symbol": "",
                        "decimals": 0,
                        "isSufficient": false
                    },
                    "0x99e743964c036bc28931fb564817db428aa7f752": {
                        "token": "0x99e743964c036bc28931fb564817db428aa7f752",
                        "name": "",
                        "minimumBalance": 1n,
                        "symbol": "",
                        "decimals": 0,
                        "isSufficient": false
                    },
                    "0xfff9976782d46cc05630d1f6ebab18b2324d6b14": {
                        "token": "0xfff9976782d46cc05630d1f6ebab18b2324d6b14",
                        "name": "Wrapped Ether",
                        "minimumBalance": 15000000000000n,
                        "symbol": "WETH",
                        "decimals": 18,
                        "isSufficient": true
                    }
                },
                "estimatedExecutionFeeDOT": 0n,
                "estimatedDeliveryFeeDOT": 0n
            },
            "2043": {
                "parachainId": 2043,
                "features": {
                    "hasPalletXcm": true,
                    "hasDryRunApi": true,
                    "hasTxPaymentApi": true,
                    "hasDryRunRpc": true,
                    "hasDotBalance": false,
                    "hasEthBalance": false,
                    "hasXcmPaymentApi": true,
                    "supportsAliasOrigin": false,
                    "xcmVersion": "v4",
                    "supportsV2": false
                },
                "info": {
                    "tokenSymbols": "NEURO",
                    "tokenDecimals": 12,
                    "ss58Format": 101,
                    "isEthereum": false,
                    "accountType": "AccountId32",
                    "name": "Neuro Testnet",
                    "specName": "origintrail-parachain",
                    "specVersion": 151
                },
                "assets": {
                    "0xef32abea56beff54f61da319a7311098d6fbcea9": {
                        "token": "0xef32abea56beff54f61da319a7311098d6fbcea9",
                        "name": "Trac",
                        "minimumBalance": 1000000000000000n,
                        "symbol": "TRAC",
                        "decimals": 18,
                        "isSufficient": true
                    }
                },
                "estimatedExecutionFeeDOT": 306833n,
                "estimatedDeliveryFeeDOT": 307250000n
            },
            "3369": {
                "parachainId": 3369,
                "features": {
                    "hasPalletXcm": true,
                    "hasDryRunApi": true,
                    "hasTxPaymentApi": true,
                    "hasDryRunRpc": true,
                    "hasDotBalance": false,
                    "hasEthBalance": false,
                    "hasXcmPaymentApi": true,
                    "supportsAliasOrigin": true,
                    "xcmVersion": "v5",
                    "supportsV2": false
                },
                "info": {
                    "tokenSymbols": "MUSE",
                    "tokenDecimals": 18,
                    "ss58Format": 29972,
                    "isEthereum": true,
                    "accountType": "AccountId20",
                    "name": "Muse Testnet",
                    "specName": "muse",
                    "specVersion": 1029
                },
                "assets": {
                    "0xb34a6924a02100ba6ef12af1c798285e8f7a16ee": {
                        "token": "0xb34a6924a02100ba6ef12af1c798285e8f7a16ee",
                        "name": "Muse",
                        "minimumBalance": 10000000000000000n,
                        "symbol": "MUSE",
                        "decimals": 18,
                        "isSufficient": true
                    }
                },
                "estimatedExecutionFeeDOT": 1000000000n,
                "estimatedDeliveryFeeDOT": 306650000n
            }
        }
    }
} as const
export default registry
