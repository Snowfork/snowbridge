import { Keyring } from "@polkadot/keyring"
import { Context, contextConfigFor, environment, toPolkadotV2 } from "@snowbridge/api"
import { formatEther, Wallet } from "ethers"
import { cryptoWaitReady } from "@polkadot/util-crypto"
import { assetRegistryFor } from "@snowbridge/registry"
import { WETH9__factory } from "@snowbridge/contract-types"

export const transferToPolkadot = async (
    destinationChainId: number,
    symbol: string,
    amount: bigint
) => {
    await cryptoWaitReady()

    let env = "local_e2e"
    if (process.env.NODE_ENV !== undefined) {
        env = process.env.NODE_ENV
    }
    console.log(`Using environment '${env}'`)

    const context = new Context(contextConfigFor(env))

    const ETHEREUM_ACCOUNT = new Wallet(
        process.env.ETHEREUM_KEY ??
            "0x5e002a1af63fd31f1c25258f3082dc889762664cb8f218d86da85dff8b07b342",
        context.ethereum()
    )
    const ETHEREUM_ACCOUNT_PUBLIC = await ETHEREUM_ACCOUNT.getAddress()

    const polkadot_keyring = new Keyring({ type: "sr25519" })
    const POLKADOT_ACCOUNT = polkadot_keyring.addFromUri(process.env.SUBSTRATE_KEY ?? "//Ferdie")
    const POLKADOT_ACCOUNT_PUBLIC = POLKADOT_ACCOUNT.address

    console.log("eth", ETHEREUM_ACCOUNT_PUBLIC, "sub", POLKADOT_ACCOUNT_PUBLIC)

    const registry = assetRegistryFor(env)

    const assets = registry.ethereumChains[registry.ethChainId].assets
    const TOKEN_CONTRACT = Object.keys(assets)
        .map((t) => assets[t])
        .find((asset) => asset.symbol.toLowerCase().startsWith(symbol.toLowerCase()))?.token
    if (!TOKEN_CONTRACT) {
        console.log("no token contract exists, check it and rebuild asset registry.")
        return
    }

    if (symbol.toLowerCase().startsWith("weth")) {
        console.log("# Deposit and Approve WETH")
        {
            const weth9 = WETH9__factory.connect(TOKEN_CONTRACT, ETHEREUM_ACCOUNT)
            const depositResult = await weth9.deposit({ value: amount })
            const depositReceipt = await depositResult.wait()

            const approveResult = await weth9.approve(context.config.appContracts.gateway, amount)
            const approveReceipt = await approveResult.wait()

            console.log("deposit tx", depositReceipt?.hash, "approve tx", approveReceipt?.hash)
        }
    }

    console.log("# Ethereum to Asset Hub")
    {
        // Step 1. Get the delivery fee for the transaction
        const fee = await toPolkadotV2.getDeliveryFee(
            {
                gateway: context.gateway(),
                assetHub: await context.assetHub(),
                destination: await context.parachain(destinationChainId),
            },
            registry,
            // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
            TOKEN_CONTRACT!,
            destinationChainId
        )

        // Step 2. Create a transfer tx
        const transfer = await toPolkadotV2.createTransfer(
            registry,
            ETHEREUM_ACCOUNT_PUBLIC,
            POLKADOT_ACCOUNT_PUBLIC,
            // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
            TOKEN_CONTRACT!,
            destinationChainId,
            amount,
            fee
        )

        // Step 3. Validate the transaction.
        const validation = await toPolkadotV2.validateTransfer(
            {
                ethereum: context.ethereum(),
                gateway: context.gateway(),
                bridgeHub: await context.bridgeHub(),
                assetHub: await context.assetHub(),
                destParachain:
                    destinationChainId !== 1000
                        ? await context.parachain(destinationChainId)
                        : undefined,
            },
            transfer
        )
        console.log("validation result", validation)

        // Step 4. Check validation logs for errors
        if (validation.logs.find((l) => l.kind == toPolkadotV2.ValidationKind.Error)) {
            throw Error(`validation has one of more errors.`)
        }

        // Step 5. Estimate the cost of the execution cost of the transaction
        const {
            tx,
            computed: { totalValue },
        } = transfer
        const estimatedGas = await context.ethereum().estimateGas(tx)
        const feeData = await context.ethereum().getFeeData()
        const executionFee = (feeData.gasPrice ?? 0n) * estimatedGas

        console.log("tx:", tx)
        console.log("feeData:", feeData.toJSON())
        console.log("gas:", estimatedGas)
        console.log("delivery cost:", formatEther(fee.totalFeeInWei))
        console.log("execution cost:", formatEther(executionFee))
        console.log("total cost:", formatEther(fee.totalFeeInWei + executionFee))
        console.log("ether sent:", formatEther(totalValue - fee.totalFeeInWei))
        console.log("dry run:", await context.ethereum().call(tx))

        if (process.env["DRY_RUN"] != "true") {
            // Step 6. Submit the transaction
            const response = await ETHEREUM_ACCOUNT.sendTransaction(tx)
            const receipt = await response.wait(1)
            if (!receipt) {
                throw Error(`Transaction ${response.hash} not included.`)
            }

            // Step 7. Get the message receipt for tracking purposes
            const message = await toPolkadotV2.getMessageReceipt(receipt)
            if (!message) {
                throw Error(`Transaction ${receipt.hash} did not emit a message.`)
            }
            console.log(
                `Success message with message id: ${message.messageId}
                block number: ${message.blockNumber}
                tx hash: ${message.txHash}`
            )
        }
    }
    await context.destroyContext()
}
