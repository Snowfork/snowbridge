import { Keyring } from "@polkadot/keyring"
import { Context, toPolkadotSnowbridgeV2, contextConfigFor, toPolkadotV2 } from "@snowbridge/api"
import { cryptoWaitReady } from "@polkadot/util-crypto"
import { formatEther, Wallet } from "ethers"
import { assetRegistryFor } from "@snowbridge/registry"

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

    const context = new Context(contextConfigFor(env))

    const polkadot_keyring = new Keyring({ type: "sr25519" })

    const ETHEREUM_ACCOUNT = new Wallet(
        process.env.ETHEREUM_KEY ??
            "0x5e002a1af63fd31f1c25258f3082dc889762664cb8f218d86da85dff8b07b342",
        context.ethChain(l2ChainId),
    )
    const ETHEREUM_ACCOUNT_PUBLIC = await ETHEREUM_ACCOUNT.getAddress()
    const POLKADOT_ACCOUNT = polkadot_keyring.addFromUri(process.env.SUBSTRATE_KEY ?? "//Ferdie")
    const POLKADOT_ACCOUNT_PUBLIC = POLKADOT_ACCOUNT.address

    console.log("eth", ETHEREUM_ACCOUNT_PUBLIC, "sub", POLKADOT_ACCOUNT_PUBLIC)

    const registry = assetRegistryFor(env)

    const assets = registry.ethereumChains[l2ChainId].assets
    const TOKEN_CONTRACT = Object.keys(assets)
        .map((t) => assets[t])
        .find((asset) => asset.symbol.toLowerCase().startsWith(symbol.toLowerCase()))?.token
    if (!TOKEN_CONTRACT) {
        console.error("no token contract exists, check it and rebuild asset registry.")
        throw Error(`No token found for ${symbol}`)
    }

    console.log("TOKEN_CONTRACT", TOKEN_CONTRACT)

    console.log("Ethereum to Polkadot")
    {
        // Step 0. Create a transfer implementation
        const transferImpl = toPolkadotSnowbridgeV2.createL2TransferImplementation(
            l2ChainId,
            destParaId,
            registry,
            TOKEN_CONTRACT,
        )
        // Step 1. Get the delivery fee for the transaction
        let fee = await transferImpl.getDeliveryFee(
            context,
            registry,
            l2ChainId,
            TOKEN_CONTRACT,
            amount,
            destParaId,
        )

        console.log("fee: ", fee)
        // Step 2. Create a transfer tx
        const transfer = await transferImpl.createTransfer(
            context,
            registry,
            l2ChainId,
            TOKEN_CONTRACT,
            amount,
            destParaId,
            ETHEREUM_ACCOUNT_PUBLIC,
            POLKADOT_ACCOUNT_PUBLIC,
            fee,
        )

        // Step 3. Validate the transaction.
        const validation = await transferImpl.validateTransfer(context, transfer)
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
        const estimatedGas = await context.ethChain(l2ChainId).estimateGas(tx)
        const feeData = await context.ethChain(l2ChainId).getFeeData()
        const executionFee = (feeData.gasPrice ?? 0n) * estimatedGas

        console.log("tx:", tx)
        console.log("feeData:", feeData.toJSON())
        console.log("gas:", estimatedGas)
        console.log("relayer fee:", formatEther(fee.relayerFee))
        console.log("execution cost:", formatEther(executionFee))
        console.log("total cost:", formatEther(fee.totalFeeInWei + executionFee))
        console.log("ether sent:", formatEther(totalValue - fee.totalFeeInWei))
        console.log("dry run:", await context.ethereum().call(tx))

        if (process.env["DRY_RUN"] != "true") {
            console.log("sending tx")
            // Step 5. Submit the transaction
            const response = await ETHEREUM_ACCOUNT.sendTransaction(tx)
            console.log("sent transaction")
            const receipt = await response.wait(1)
            console.log("got receipt")
            if (!receipt || receipt.status != 1) {
                throw Error(`Transaction ${response.hash} not included.`)
            }

            console.log(
                `Success messages:
                block number: ${receipt.blockNumber}
                tx hash: ${receipt.hash}`,
            )
        }
    }
    await context.destroyContext()
}
