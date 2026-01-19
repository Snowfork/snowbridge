import { Keyring } from "@polkadot/keyring"
import { Context, toPolkadotSnowbridgeV2, toPolkadotV2 } from "@snowbridge/api"
import { cryptoWaitReady } from "@polkadot/util-crypto"
import { formatEther, Wallet } from "ethers"
import { assetRegistryFor, environmentFor } from "@snowbridge/registry"
import { WETH9__factory } from "@snowbridge/contract-types"

export const transferToPolkadot = async (destParaId: number, symbol: string, amount: bigint) => {
    await cryptoWaitReady()

    let env = "local_e2e"
    if (process.env.NODE_ENV !== undefined) {
        env = process.env.NODE_ENV
    }
    console.log(`Using environment '${env}'`)

    const context = new Context(environmentFor(env))

    const polkadot_keyring = new Keyring({ type: "sr25519" })

    const ETHEREUM_ACCOUNT = new Wallet(
        process.env.ETHEREUM_KEY ?? "Your Key Goes Here",
        context.ethereum(),
    )
    const ETHEREUM_ACCOUNT_PUBLIC = await ETHEREUM_ACCOUNT.getAddress()
    const POLKADOT_ACCOUNT = polkadot_keyring.addFromUri(process.env.SUBSTRATE_KEY ?? "//Ferdie")
    const POLKADOT_ACCOUNT_PUBLIC = POLKADOT_ACCOUNT.address

    console.log("eth", ETHEREUM_ACCOUNT_PUBLIC, "sub", POLKADOT_ACCOUNT_PUBLIC)

    const registry = assetRegistryFor(env)

    const assets = registry.ethereumChains[registry.ethChainId].assets
    const TOKEN_CONTRACT = Object.keys(assets)
        .map((t) => assets[t])
        .find((asset) => asset.symbol.toLowerCase().startsWith(symbol.toLowerCase()))?.token
    if (!TOKEN_CONTRACT) {
        console.error("no token contract exists, check it and rebuild asset registry.")
        throw Error(`No token found for ${symbol}`)
    }

    console.log("TOKEN_CONTRACT", TOKEN_CONTRACT)
    if (symbol.toLowerCase().startsWith("weth")) {
        console.log("# Deposit and Approve WETH")
        {
            const weth9 = WETH9__factory.connect(TOKEN_CONTRACT, ETHEREUM_ACCOUNT)
            const depositResult = await weth9.deposit({ value: amount })
            const depositReceipt = await depositResult.wait()

            const approveResult = await weth9.approve(context.environment.gatewayContract, amount)
            const approveReceipt = await approveResult.wait()

            console.log("deposit tx", depositReceipt?.hash, "approve tx", approveReceipt?.hash)
        }
    }

    console.log("Ethereum to Polkadot")
    {
        // Step 0. Create a transfer implementation
        const transferImpl = toPolkadotSnowbridgeV2.createTransferImplementation(
            destParaId,
            registry,
            TOKEN_CONTRACT,
        )
        // Step 1. Get the delivery fee for the transaction
        let fee = await transferImpl.getDeliveryFee(context, registry, TOKEN_CONTRACT, destParaId)

        console.log("fee: ", fee)
        // Step 2. Create a transfer tx
        const transfer = await transferImpl.createTransfer(
            {
                ethereum: context.ethereum(),
                assetHub: await context.assetHub(),
                destination:
                    destParaId !== registry.assetHubParaId
                        ? await context.parachain(destParaId)
                        : undefined,
            },
            registry,
            destParaId,
            ETHEREUM_ACCOUNT_PUBLIC,
            "0x460411e07f93dc4bc2b3a6cb67dad89ca26e8a54054d13916f74c982595c2e0e",
            TOKEN_CONTRACT,
            amount,
            fee,
        )

        // Step 3. Validate the transaction.
        const validation = await transferImpl.validateTransfer(
            {
                ethereum: context.ethereum(),
                gateway: context.gatewayV2(),
                bridgeHub: await context.bridgeHub(),
                assetHub: await context.assetHub(),
                destination:
                    destParaId !== registry.assetHubParaId
                        ? await context.parachain(destParaId)
                        : undefined,
            },
            transfer,
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
            if (!receipt) {
                throw Error(`Transaction ${response.hash} not included.`)
            }

            // Step 7. Get the message receipt for tracking purposes
            const message = await toPolkadotSnowbridgeV2.getMessageReceipt(receipt)
            if (!message) {
                throw Error(`Transaction ${receipt.hash} did not emit a message.`)
            }
            console.log(
                `Success message with nonce: ${message.nonce}
                block number: ${message.blockNumber}
                tx hash: ${message.txHash}`,
            )
        }
    }
    await context.destroyContext()
}
