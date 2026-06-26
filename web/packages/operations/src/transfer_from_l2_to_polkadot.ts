import { Keyring } from "@polkadot/keyring"
import { Context, createApi } from "@snowbridge/api"
import { assetsV2 } from "@snowbridge/api"
import { EthersEthereumProvider, EthersProviderTypes } from "@snowbridge/provider-ethers"
import { cryptoWaitReady } from "@polkadot/util-crypto"
import { formatEther, Wallet } from "ethers"
import { bridgeInfoFor } from "@snowbridge/registry"
import { IERC20__factory } from "@snowbridge/contract-types"
import { findFeeBreakdownTotal, findFeeTotal } from "./fee"

export const transferToPolkadot = async (
    l2ChainId: number,
    destParaId: number,
    symbol: string,
    amount: bigint,
) => {
    await cryptoWaitReady()

    let env = "local_e2e"
    if (process.env.NODE_ENV !== undefined) {
        env = process.env.NODE_ENV
    }
    console.log(`Using environment '${env}'`)

    // Repro mode for the WETH validation false-pass: approve the adaptor for
    // exactly the amount the (unpatched) validation checks against, which is
    // less than the amount the adaptor's safeTransferFrom actually pulls. This
    // forces the allowance into the [validationChecks, adaptorPulls) window so
    // the bug is deterministic. Implies dry-run so no funds are spent.
    const REPRO_TIGHT_ALLOWANCE = process.env["REPRO_TIGHT_ALLOWANCE"] === "true"
    const dryRun = REPRO_TIGHT_ALLOWANCE || process.env["DRY_RUN"] === "true"

    const info = bridgeInfoFor(env)
    const { registry } = info
    const api = createApi({
        info,
        ethereumProvider: new EthersEthereumProvider(),
    })
    const context: Context<EthersProviderTypes> = api.context

    const polkadot_keyring = new Keyring({ type: "sr25519" })

    const ETHEREUM_ACCOUNT = new Wallet(
        process.env.ETHEREUM_KEY ?? "Your Key Goes Here",
        context.ethChain(l2ChainId),
    )
    const ETHEREUM_ACCOUNT_PUBLIC = await ETHEREUM_ACCOUNT.getAddress()
    const POLKADOT_ACCOUNT = polkadot_keyring.addFromUri(process.env.SUBSTRATE_KEY ?? "//Ferdie")
    const POLKADOT_ACCOUNT_PUBLIC = POLKADOT_ACCOUNT.address

    console.log("eth", ETHEREUM_ACCOUNT_PUBLIC, "sub", POLKADOT_ACCOUNT_PUBLIC)

    const assets = registry.ethereumChains[`ethereum_l2_${l2ChainId}`].assets
    const TOKEN_CONTRACT = Object.keys(assets)
        .map((t) => assets[t])
        .find((asset) => asset.symbol.toLowerCase().startsWith(symbol.toLowerCase()))?.token
    if (!TOKEN_CONTRACT) {
        console.error("no token contract exists, check it and rebuild asset registry.")
        throw Error(`No token found for ${symbol}`)
    }

    console.log("TOKEN_CONTRACT", TOKEN_CONTRACT)

    console.log("Ethereum to Polkadot")

    // Step 0. Create a transfer implementation
    const transferImpl = api.sender(
        { kind: "ethereum_l2", id: l2ChainId },
        { kind: "polkadot", id: destParaId },
    )

    // Step 1. Get the delivery fee for the transaction
    let fee = await transferImpl.fee(TOKEN_CONTRACT, amount)
    console.log("fee: ", fee)

    // Step 2. Create a transfer tx (needed before approve so we can read the
    // exact value the adaptor will pull, transfer.computed.totalValue).
    const transfer = await transferImpl.tx(
        ETHEREUM_ACCOUNT_PUBLIC,
        POLKADOT_ACCOUNT_PUBLIC,
        TOKEN_CONTRACT,
        amount,
        fee,
    )

    // The adaptor pulls exactly depositParams.inputAmount (= totalValue) from the
    // user. The unpatched validation only checks balance/allowance against
    // totalValue - totalFeeInWei (= amount + Across origin fee), so an allowance
    // in between passes validation and then reverts on-chain.
    const adaptorPulls = transfer.computed.totalValue
    const validationChecks = adaptorPulls - findFeeTotal(fee, "ETH")

    // Step 3. Approve the adaptor as spender (ERC20 inputs only).
    if (TOKEN_CONTRACT != assetsV2.ETHER_TOKEN_ADDRESS) {
        console.log("# Approve")
        const erc20 = IERC20__factory.connect(TOKEN_CONTRACT, ETHEREUM_ACCOUNT)
        const l2AdapterAddress = context.environment.l2Bridge?.l2Chains[l2ChainId].adapterAddress
        if (!l2AdapterAddress) {
            throw new Error("L2 bridge configuration is missing.")
        }

        if (REPRO_TIGHT_ALLOWANCE) {
            console.log(
                `REPRO: validation checks allowance >= ${validationChecks.toString()}, ` +
                    `but the adaptor pulls ${adaptorPulls.toString()} ` +
                    `(gap ${(adaptorPulls - validationChecks).toString()} = totalFeeInWei). ` +
                    `Approving exactly ${validationChecks.toString()}.`,
            )
            // Reset to 0 then set the tight allowance.
            const resetTx = await erc20.approve(l2AdapterAddress, 0n)
            await resetTx.wait()
            const approveTx = await erc20.approve(l2AdapterAddress, validationChecks)
            await approveTx.wait()
            const newAllowance = await erc20.allowance(ETHEREUM_ACCOUNT_PUBLIC, l2AdapterAddress)
            console.log("allowance set to", newAllowance.toString())
        } else {
            const allowance = await erc20.allowance(ETHEREUM_ACCOUNT_PUBLIC, l2AdapterAddress)
            if (allowance <= amount) {
                // Step 1: Reset allowance to 0 (required by this ERC20 implementation)
                console.log("Resetting allowance to 0...")
                const resetTx = await erc20.approve(l2AdapterAddress, 0n)
                await resetTx.wait()

                // Step 2: Set new allowance (higher than transfer amount for gateway fees)
                const approveAmount = amount * 10n // 10x buffer
                console.log("Setting new allowance to", approveAmount.toString())
                const approveTx = await erc20.approve(l2AdapterAddress, approveAmount)
                await approveTx.wait()

                const newAllowance = await erc20.allowance(ETHEREUM_ACCOUNT_PUBLIC, l2AdapterAddress)
                console.log("newAllowance", newAllowance.toString())
            }
        }
    }

    {
        // Step 4. Validate the transaction.
        const validation = await transferImpl.validate(transfer)
        console.log("validation result", validation)

        // Step 5. Check validation logs for errors
        if (!validation.success) {
            if (REPRO_TIGHT_ALLOWANCE) {
                const allowanceRejected = validation.logs.some((l: { message: string }) =>
                    l.message.includes("approved as a spender"),
                )
                const gasEstimationFailed = validation.logs.some((l: { message: string }) =>
                    l.message.includes("estimate gas"),
                )
                if (allowanceRejected) {
                    console.log(
                        "REPRO RESULT (patched): validation REJECTED on the allowance check " +
                            "(GatewaySpenderLimitReached) — the gap is caught directly with a " +
                            "clear, actionable error.",
                    )
                } else if (gasEstimationFailed) {
                    console.log(
                        "REPRO RESULT (unpatched): the static balance/allowance checks PASSED " +
                            "(the under-count false-pass), but validate()'s estimateGas dry-run " +
                            "caught the revert and failed with a cryptic gas-estimation error. " +
                            "So the bug is backstopped by estimateGas — the harm is a confusing " +
                            "message, not an uncaught on-chain revert.",
                    )
                } else {
                    console.log(
                        "REPRO INCONCLUSIVE: validation was rejected for an unrelated reason " +
                            "(e.g. insufficient WETH balance). Ensure WETH balance >= value so " +
                            "the allowance is the binding constraint. Logs: " +
                            JSON.stringify(validation.logs),
                    )
                }
            }
            throw Error(`validation has one of more errors.` + JSON.stringify(validation.logs))
        }

        if (REPRO_TIGHT_ALLOWANCE) {
            console.log(
                "REPRO: validation PASSED with the tight allowance. Confirming the on-chain " +
                    "call reverts (this is the bug on unpatched code)...",
            )
        }

        // Step 6. Estimate the cost of the execution cost of the transaction
        const {
            tx,
            computed: { totalValue },
        } = transfer
        const estimatedGas = await context.ethereumProvider.estimateGas(
            context.ethChain(l2ChainId),
            tx,
        )
        const feeData = await context.ethereumProvider.getFeeData(context.ethChain(l2ChainId))
        const executionFee = (feeData.gasPrice ?? 0n) * estimatedGas
        const relayerFee = findFeeBreakdownTotal(fee, "relayer", "ETH")
        const deliveryFee = findFeeTotal(fee, "ETH")

        console.log("tx:", tx)
        console.log("feeData:", feeData)
        console.log("gas:", estimatedGas)
        console.log("relayer fee:", formatEther(relayerFee))
        console.log("execution cost:", formatEther(executionFee))
        console.log("total cost:", formatEther(deliveryFee + executionFee))
        console.log("ether sent:", formatEther(totalValue - deliveryFee))
        try {
            console.log("dry run:", await context.ethChain(l2ChainId).call(tx))
            if (REPRO_TIGHT_ALLOWANCE) {
                console.log("REPRO: on-chain call unexpectedly succeeded.")
            }
        } catch (e) {
            if (REPRO_TIGHT_ALLOWANCE) {
                console.log(
                    "REPRO RESULT: validation PASSED but the on-chain call REVERTED (the bug). " +
                        "Reason:",
                    (e as Error).message,
                )
                await context.destroyContext()
                return
            }
            throw e
        }

        if (!dryRun) {
            console.log("sending tx")
            // Step 7. Submit the transaction
            const response = await ETHEREUM_ACCOUNT.sendTransaction(tx)
            console.log("sent transaction")
            const receipt = await response.wait(1)
            console.log("got receipt")
            if (!receipt || receipt.status != 1) {
                throw Error(`Transaction ${response.hash} not included.`)
            }

            const message = await transferImpl.messageId(receipt)
            if (!message) {
                throw Error(`Transaction ${receipt.hash} did not emit a message.`)
            }

            console.log(
                `Success message with id: ${message.messageId}
                deposit id: ${message.depositId}
                block number: ${receipt.blockNumber}
                tx hash: ${receipt.hash}`,
            )
        }
    }
    await context.destroyContext()
}
