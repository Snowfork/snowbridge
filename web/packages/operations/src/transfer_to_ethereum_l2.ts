import { Keyring } from "@polkadot/keyring"
import { Context, toEthereumSnowbridgeV2, toEthereumV2 } from "@snowbridge/api"
import { cryptoWaitReady } from "@polkadot/util-crypto"
import { formatUnits, Wallet } from "ethers"
import { bridgeInfoFor } from "@snowbridge/registry"
import { ContractCall } from "../../base-types/dist"

export const transferToEthereumL2 = async (
    sourceParaId: number,
    l2ChainId: number,
    symbol: string,
    amount: bigint,
    options?: {
        feeTokenLocation?: any
        contractCall?: ContractCall
    },
) => {
    await cryptoWaitReady()

    let env = "local_e2e"
    if (process.env.NODE_ENV !== undefined) {
        env = process.env.NODE_ENV
    }
    console.log(`Using environment '${env}'`)

    const { registry, environment } = bridgeInfoFor(env)
    const context = new Context(environment)

    const polkadot_keyring = new Keyring({ type: "sr25519" })

    const ETHEREUM_ACCOUNT = new Wallet(
        process.env.ETHEREUM_KEY ?? "Your Key Goes Here",
        context.ethereum(),
    )
    const ETHEREUM_ACCOUNT_PUBLIC = await ETHEREUM_ACCOUNT.getAddress()
    const POLKADOT_ACCOUNT = polkadot_keyring.addFromUri(process.env.SUBSTRATE_KEY ?? "//Ferdie")
    const POLKADOT_ACCOUNT_PUBLIC = POLKADOT_ACCOUNT.address

    console.log("eth", ETHEREUM_ACCOUNT_PUBLIC, "sub", POLKADOT_ACCOUNT_PUBLIC)

    const assets = registry.ethereumChains[`ethereum_${registry.ethChainId}`].assets
    const TOKEN_CONTRACT = Object.keys(assets)
        .map((t) => assets[t])
        .find((asset) => asset.symbol.toLowerCase().startsWith(symbol.toLowerCase()))?.token
    if (!TOKEN_CONTRACT) {
        console.error("no token contract exists, check it and rebuild asset registry.")
        throw Error(`No token found for ${symbol}`)
    }

    console.log("Asset Hub to Ethereum")
    {
        // Step 0. Create a transfer implementation
        const transferImpl = toEthereumSnowbridgeV2.createL2TransferImplementation(
            sourceParaId,
            registry,
            TOKEN_CONTRACT,
        )
        // Step 1. Get the delivery fee for the transaction
        let fee: toEthereumV2.DeliveryFee = await transferImpl.getDeliveryFee(
            context,
            registry,
            l2ChainId,
            TOKEN_CONTRACT,
            amount,
        )

        // Step 2. Create a transfer tx
        const transfer = await transferImpl.createTransfer(
            context,
            registry,
            l2ChainId,
            TOKEN_CONTRACT,
            amount,
            POLKADOT_ACCOUNT_PUBLIC,
            ETHEREUM_ACCOUNT_PUBLIC,
            fee,
            options,
        )

        // Step 3. Estimate the cost of the execution cost of the transaction
        console.log("call: ", transfer.tx.inner.toHex())
        const feePayment = (
            await transfer.tx.paymentInfo(POLKADOT_ACCOUNT, { withSignedTransaction: true })
        ).toPrimitive() as any
        console.log(
            `execution fee (${transfer.computed.sourceParachain.info.tokenSymbols}):`,
            formatUnits(
                feePayment.partialFee,
                transfer.computed.sourceParachain.info.tokenDecimals,
            ),
        )
        console.log(
            `delivery fee (${registry.parachains[`polkadot_${registry.assetHubParaId}`].info.tokenSymbols}): `,
            formatUnits(fee.totalFeeInDot, transfer.computed.sourceParachain.info.tokenDecimals),
        )

        // Step 4. Validate the transaction.
        const validation = await transferImpl.validateTransfer(context, transfer)
        console.log("validation result", validation)

        // Step 5. Check validation logs for errors
        if (validation.logs.find((l) => l.kind == toEthereumSnowbridgeV2.ValidationKind.Error)) {
            throw Error(`validation has one of more errors.`)
        }
        if (process.env["DRY_RUN"] != "true") {
            // Step 6. Submit transaction and get receipt for tracking
            const response = await toEthereumSnowbridgeV2.signAndSend(
                context,
                transfer,
                POLKADOT_ACCOUNT,
                {
                    withSignedTransaction: true,
                },
            )
            if (!response) {
                throw Error(`Transaction ${response} not included.`)
            }
            console.log(
                `Success message with message id: ${response.messageId}
                block number: ${response.blockNumber}
                tx hash: ${response.txHash}`,
            )
        }
    }
    await context.destroyContext()
}
