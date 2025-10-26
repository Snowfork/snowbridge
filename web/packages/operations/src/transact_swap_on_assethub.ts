import "dotenv/config"
import { Keyring } from "@polkadot/keyring"
import { Context, toPolkadotSnowbridgeV2, contextConfigFor } from "@snowbridge/api"
import { cryptoWaitReady } from "@polkadot/util-crypto"
import { formatEther, Wallet } from "ethers"
import { assetRegistryFor } from "@snowbridge/registry"
import { ETHER_TOKEN_ADDRESS } from "@snowbridge/api/src/assets_v2"
import {accountToLocation, erc20Location, ethereumNetwork} from "@snowbridge/api/src/xcmBuilder"

const ASSET_HUB_PARA_ID = 1000
const DOT_EXCHANGE_RATE = 0.00087 // 1 DOT = 0.00077 ETH
const USDC_EXCHANGE_RATE = 0.14 // 1 DOT = 0.14 USDC (approximate, adjust based on current rates)
const USDC_TO_ETH_RATE = 0.001 // 1 USDC ≈ 0.001 ETH (approximate $1 USDC, $1000 ETH)
const USDC_TOKEN_ADDRESS = "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48" // USDC on Ethereum mainnet

export const transactSwapOnAssetHub = async (etherAmount: bigint) => {
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
    // ETH has 18 decimals, DOT has 10 decimals
    // Formula: (etherAmount in wei / (rate * 1e18)) * 1e10
    const exchangeRateScaled = BigInt(Math.floor(DOT_EXCHANGE_RATE * 1e18))
    const swapEtherAmount = etherAmount - 1000000000000000n
    const expectedDotAmount = (swapEtherAmount * BigInt(1e10)) / exchangeRateScaled

    // Reserve some DOT for remote fees (keep it as DOT, don't swap)
    const dotReservedForFees = BigInt(20_000_000_000) // 2 DOT in plancks (10 decimals)

    // Calculate DOT available for USDC swap (total DOT - DOT reserved for fees)
    const dotForUsdcSwap = expectedDotAmount > dotReservedForFees ? expectedDotAmount - dotReservedForFees : 0n

    // Use half of remaining DOT for remote fees, keep the other half on AH for delivery to BH
    const dotForRemoteFees = dotReservedForFees / 2n

    // Calculate expected USDC amount from remaining DOT
    // USDC exchange rate: 1 DOT = USDC_EXCHANGE_RATE USDC (e.g., 0.14 USDC)
    // USDC has 6 decimals, DOT has 10 decimals
    const expectedUsdcAmount = (expectedDotAmount * BigInt(Math.floor(USDC_EXCHANGE_RATE * 1e6))) / BigInt(1e10)

    // Reserve some USDC to swap back to ETHER for remote fees
    // Direct USDC → ETHER swap: 1 USDC ≈ 0.001 ETH
    // For 0.001 ETH, we need: 0.001 / 0.001 = 1 USDC
    const etherReservedForFees = BigInt(1000000000000000) // 0.001 ETH in wei (18 decimals)
    const usdcRateScaled = BigInt(Math.floor(USDC_TO_ETH_RATE * 1e18))  // 0.001 ETH per USDC, scaled
    const usdcToSwapBackToEther = (etherReservedForFees * BigInt(1e6)) / usdcRateScaled  // USDC needed (6 decimals)

    // Calculate USDC available for transfer (total USDC - USDC to swap back)
    const usdcForTransfer = expectedUsdcAmount > usdcToSwapBackToEther ? expectedUsdcAmount - usdcToSwapBackToEther : 0n

    // Helper function to format USDC with proper decimals
    const formatUsdc = (usdcUnits: bigint): string => {
        const usdc = Number(usdcUnits) / 1e6
        return usdc.toFixed(6)
    }

    // Helper function to format DOT with proper decimals
    const formatDot = (plancks: bigint): string => {
        const dot = Number(plancks) / 1e10
        return dot.toFixed(4)
    }

    console.log("Transacting to Asset Hub with custom XCM to swap ETHER → DOT → USDC → ETHER → Ethereum")
    console.log(`  ETHER amount: ${formatEther(etherAmount)} ETH`)
    console.log(`  USDC token address: ${USDC_TOKEN_ADDRESS}`)
    console.log(`  DOT exchange rate: 1 DOT = ${DOT_EXCHANGE_RATE} ETH`)
    console.log(`  Expected DOT from ETHER swap: ${formatDot(expectedDotAmount)} DOT`)
    console.log(`  USDC exchange rate: 1 DOT = ${USDC_EXCHANGE_RATE} USDC`)
    console.log(`  Expected USDC from DOT swap: ${formatUsdc(expectedUsdcAmount)} USDC (${expectedUsdcAmount} units)`)
    console.log(`  USDC→ETH rate: 1 USDC = ${USDC_TO_ETH_RATE} ETH`)
    console.log(`  DOT available (not swapped to USDC): ${formatDot(dotForUsdcSwap)} DOT`)
    console.log(`  DOT for remote fees (half of available): ${formatDot(dotForRemoteFees)} DOT`)
    console.log(`  USDC for transfer: ${formatUsdc(usdcForTransfer)} USDC (${usdcForTransfer} units)`)
    {
        // Build the custom XCM instructions
        const etherLocation = erc20Location(registry.ethChainId, ETHER_TOKEN_ADDRESS)
        const usdcLocation = erc20Location(registry.ethChainId, USDC_TOKEN_ADDRESS)

        const customXcm = [
            // Step 1: Swap all ETHER for DOT
            {
                exchangeAsset: {
                    give: {
                        definite: [
                            {
                                id: etherLocation,
                                fun: {
                                    Fungible: swapEtherAmount,
                                },
                            },
                        ],
                    },
                    want: [
                        {
                            id: {
                                parents: 1,
                                interior: "Here",
                            },
                            fun: {
                                Fungible: expectedDotAmount,
                            },
                        },
                    ],
                    maximal: true,
                },
            },
            // Step 2: Swap most DOT for USDC, keeping dotReservedForFees
            {
                exchangeAsset: {
                    give: {
                        definite: [
                            {
                                id: {
                                    parents: 1,
                                    interior: "Here",
                                },
                                fun: {
                                    Fungible: dotForUsdcSwap,
                                },
                            },
                        ],
                    },
                    want: [
                        {
                            id: usdcLocation,
                            fun: {
                                Fungible: expectedUsdcAmount,
                            },
                        },
                    ],
                    maximal: true,
                },
            },
            // Step 3: Send USDC back to Ethereum using Ether for remote fees
            {
                initiateTransfer: {
                    destination: {
                        parents: 2,
                        interior: {
                            x1: [
                                {
                                    GlobalConsensus: {
                                        Ethereum: {
                                            chainId: registry.ethChainId,
                                        },
                                    },
                                },
                            ],
                        },
                    },
                    remoteFees: {
                        ReserveWithdraw: {
                            definite: [
                                {
                                    id: etherLocation,
                                    fun: {
                                        Fungible: 10000000000000000n,
                                    },
                                },
                            ],
                        },
                    },
                    preserveOrigin: true,
                    assets: [
                        {
                            ReserveWithdraw: {
                                wild: {
                                    allOf: {
                                        id: usdcLocation,
                                        fun: "Fungible",
                                    },
                                },
                            },
                        },
                    ],
                    remoteXcm: [
                        {
                            depositAsset: {
                                assets: {
                                    wild: {
                                        allCounted: 2,
                                    },
                                },
                                beneficiary: {
                                    parents: 0,
                                    interior: {
                                        x1: [
                                            {
                                                AccountKey20: {
                                                    key: ETHEREUM_ACCOUNT_PUBLIC,
                                                    network: null,
                                                },
                                            },
                                        ],
                                    },
                                },
                            },
                        },
                    ],
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
            POLKADOT_ACCOUNT_PUBLIC, // Beneficiary on Asset Hub (not used since we're swapping and sending back)
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

        if (validation.data.bridgeHubDryRunError) {
            console.error("\nBridge Hub Dry Run Error:")
            console.error(validation.data.bridgeHubDryRunError)
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
        console.log("  USDC token address:", USDC_TOKEN_ADDRESS)
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

if (process.argv.length != 3) {
    console.error("Expected arguments: `etherAmount`")
    console.error("Example: npm run transactSwapOnAssetHub 1000000000000000000")
    console.error("  This will swap 1 ETH for USDC via DOT on Asset Hub")
    process.exit(1)
}

transactSwapOnAssetHub(BigInt(process.argv[2]))
    .then(() => process.exit(0))
    .catch((error) => {
        console.error("Error:", error)
        process.exit(1)
    })
