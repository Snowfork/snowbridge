import "dotenv/config"
import { Keyring } from "@polkadot/keyring"
import { Context, toPolkadotSnowbridgeV2, contextConfigFor } from "@snowbridge/api"
import { cryptoWaitReady } from "@polkadot/util-crypto"
import { formatEther, Wallet } from "ethers"
import { assetRegistryFor } from "@snowbridge/registry"
import { ETHER_TOKEN_ADDRESS } from "@snowbridge/api/src/assets_v2"
import { erc20Location } from "@snowbridge/api/src/xcmBuilder"

const ASSET_HUB_PARA_ID = 1000
const DOT_EXCHANGE_RATE = 0.00077 // 1 DOT = 0.00077 ETH
const WBTC_EXCHANGE_RATE = 0.000027 // 1 DOT = 0.000027 wBTC

export const transactSwapOnAssetHub = async (etherAmount: bigint, wbtcTokenAddress: string) => {
    await cryptoWaitReady()

    let env = "polkadot_mainnet"
    if (process.env.NODE_ENV !== undefined) {
        env = process.env.NODE_ENV
    }
    console.log(`Using environment '${env}'`)

    const context = new Context(contextConfigFor(env))

    const polkadot_keyring = new Keyring({ type: "sr25519" })

    const ETHEREUM_ACCOUNT = new Wallet(
        process.env.ETHEREUM_KEY ??
            "0x5e002a1af63fd31f1c25258f3082dc889762664cb8f218d86da85dff8b07b342",
        context.ethereum()
    )
    const ETHEREUM_ACCOUNT_PUBLIC = await ETHEREUM_ACCOUNT.getAddress()
    const POLKADOT_ACCOUNT = polkadot_keyring.addFromUri(process.env.SUBSTRATE_KEY ?? "//Ferdie")
    const POLKADOT_ACCOUNT_PUBLIC = POLKADOT_ACCOUNT.address

    console.log("eth", ETHEREUM_ACCOUNT_PUBLIC, "sub", POLKADOT_ACCOUNT_PUBLIC)

    const registry = assetRegistryFor(env)
    const assetHub = await context.assetHub()

    const TOKEN_CONTRACT = ETHER_TOKEN_ADDRESS
    const relayerFee = 100_000_000_000_000n // 0.0001 ETH

    // Calculate expected DOT amount from ETHER
    // DOT exchange rate: 1 DOT = DOT_EXCHANGE_RATE ETH (e.g., 0.00077 ETH)
    // So: expectedDOT = etherAmount / DOT_EXCHANGE_RATE
    // DOT has 10 decimals, ETH has 18 decimals
    const expectedDotAmount = (etherAmount * BigInt(1e10)) / BigInt(Math.floor(DOT_EXCHANGE_RATE * 1e18))

    // Calculate expected wBTC amount from DOT
    // wBTC exchange rate: 1 DOT = WBTC_EXCHANGE_RATE wBTC (e.g., 0.000027 wBTC)
    // So: expectedWBTC = expectedDOT * WBTC_EXCHANGE_RATE
    // wBTC has 8 decimals, DOT has 10 decimals
    const expectedWbtcAmount = (expectedDotAmount * BigInt(Math.floor(WBTC_EXCHANGE_RATE * 1e8))) / BigInt(1e10)

    console.log("Transacting to Asset Hub with custom XCM to swap ETHER → DOT → wBTC → Ethereum")
    console.log(`  ETHER amount: ${formatEther(etherAmount)} ETH`)
    console.log(`  DOT exchange rate: 1 DOT = ${DOT_EXCHANGE_RATE} ETH`)
    console.log(`  Expected DOT: ${expectedDotAmount / BigInt(1e10)} DOT (${expectedDotAmount} plancks)`)
    console.log(`  wBTC exchange rate: 1 DOT = ${WBTC_EXCHANGE_RATE} wBTC`)
    console.log(`  Expected wBTC: ${expectedWbtcAmount / BigInt(1e8)} wBTC (${expectedWbtcAmount} satoshis)`)
    {
        // Build the custom XCM instructions
        const etherLocation = erc20Location(registry.ethChainId, ETHER_TOKEN_ADDRESS)
        const wbtcLocation = erc20Location(registry.ethChainId, wbtcTokenAddress)

        const customXcm = [
            // Step 1: Swap ETHER for DOT on Asset Hub (using all available ETHER)
            {
                exchangeAsset: {
                    give: {
                        wild: {
                            allOf: {
                                id: etherLocation,
                                fun: "Fungible",
                            },
                        },
                    },
                    want: [
                        {
                            id: {
                                parents: 1,
                                interior: "Here",
                            },
                            Fungible: expectedDotAmount,
                        },
                    ],
                    maximal: true,
                },
            },
            // Step 2: Swap DOT for wBTC on Asset Hub (using all available DOT)
            {
                exchangeAsset: {
                    give: {
                        wild: {
                            allOf: {
                                id: {
                                    parents: 1,
                                    interior: "Here",
                                },
                                fun: "Fungible",
                            },
                        },
                    },
                    want: [
                        {
                            id: wbtcLocation,
                            Fungible: expectedWbtcAmount,
                        },
                    ],
                    maximal: true,
                },
            },
            // Step 3: Send wBTC back to Ethereum
            {
                initiateTransfer: {
                    destination: {
                        parents: 2,
                        interior: {
                            x2: [
                                {
                                    GlobalConsensus: {
                                        Ethereum: {
                                            chainId: registry.ethChainId,
                                        },
                                    },
                                },
                                {
                                    AccountKey20: {
                                        key: ETHEREUM_ACCOUNT_PUBLIC,
                                        network: null,
                                    },
                                },
                            ],
                        },
                    },
                    remote_fees: {
                        reserve: null,
                    },
                    preserve_origin: false,
                    assets: [
                        {
                            wild: {
                                allOf: {
                                    id: wbtcLocation,
                                    fun: "Fungible",
                                },
                            },
                        },
                    ],
                    remote_xcm: [],
                },
            },
        ]

        // Step 0. Create an ERC20ToAH transfer implementation
        const transferImpl = toPolkadotSnowbridgeV2.createTransferImplementation(
            ASSET_HUB_PARA_ID,
            registry,
            TOKEN_CONTRACT
        )

        // Step 1. Get the delivery fee for the transaction
        let fee = await transferImpl.getDeliveryFee(
            context,
            registry,
            TOKEN_CONTRACT,
            ASSET_HUB_PARA_ID,
            relayerFee,
            {
                customXcm: customXcm,
            }
        )

        console.log("fee: ", fee)

        // Step 2. Create a transfer tx with custom XCM
        const transfer = await transferImpl.createTransfer(
            context,
            registry,
            ASSET_HUB_PARA_ID,
            ETHEREUM_ACCOUNT_PUBLIC,
            ETHEREUM_ACCOUNT_PUBLIC, // Beneficiary on Asset Hub (not used since we're swapping and sending back)
            TOKEN_CONTRACT,
            etherAmount,
            fee,
            customXcm
        )

        // Step 3. Validate the transaction
        console.log("Validating transaction...")
        const validation = await transferImpl.validateTransfer(context, transfer)

        console.log("Validation result:")
        validation.logs.forEach((log) => {
            console.log(`  [${log.kind}] ${log.message}`)
        })

        // Display dry run errors if present
        if (validation.data.assetHubDryRunError) {
            console.error("\nAsset Hub Dry Run Error:")
            console.error(validation.data.assetHubDryRunError)
        }

        if (!validation.success) {
            console.error("\nValidation FAILED - transaction would likely fail")
            throw Error("Validation failed")
        }
        console.log("\nValidation SUCCESS - all checks passed")

        // Display transaction details
        console.log("\nTransaction Details:")
        console.log("  User ether balance:", formatEther(validation.data.etherBalance))
        if (validation.data.feeInfo) {
            console.log("  Estimated gas:", validation.data.feeInfo.estimatedGas.toString())
            console.log("  Gas price:", formatEther(validation.data.feeInfo.feeData.gasPrice ?? 0n), "ETH")
            console.log("  Execution cost:", formatEther(validation.data.feeInfo.executionFee))
            console.log("  Delivery cost:", formatEther(fee.totalFeeInWei))
            console.log("  Total TX cost:", formatEther(validation.data.feeInfo.totalTxCost))
        }
        console.log("  Ether to swap:", formatEther(etherAmount))
        console.log("  wBTC token address:", wbtcTokenAddress)
        console.log("  Bridge status:", validation.data.bridgeStatus.toPolkadot.outbound)

        const { tx } = transfer

        if (process.env["DRY_RUN"] != "true") {
            console.log("sending tx")
            // Submit the transaction
            const response = await ETHEREUM_ACCOUNT.sendTransaction(tx)
            console.log("sent transaction")
            const receipt = await response.wait(1)
            console.log("got receipt")
            if (!receipt) {
                throw Error(`Transaction ${response.hash} not included.`)
            }

            // Get the message receipt for tracking purposes
            const message = await toPolkadotSnowbridgeV2.getMessageReceipt(receipt)
            if (!message) {
                throw Error(`Transaction ${receipt.hash} did not emit a message.`)
            }
            console.log(
                `Success message with nonce: ${message.nonce}
                block number: ${message.blockNumber}
                tx hash: ${message.txHash}`
            )
        }
    }
    await context.destroyContext()
}

if (process.argv.length != 4) {
    console.error("Expected arguments: `etherAmount, wbtcTokenAddress`")
    console.error(
        'Example: npm run transactSwapOnAssetHub 1000000000000000000 "0x2260fac5e5542a773aa44fbcfedf7c193bc2c599"'
    )
    process.exit(1)
}

transactSwapOnAssetHub(BigInt(process.argv[2]), process.argv[3])
    .then(() => process.exit(0))
    .catch((error) => {
        console.error("Error:", error)
        process.exit(1)
    })
