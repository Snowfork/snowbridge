import { Keyring } from "@polkadot/keyring"
import { createApi } from "@snowbridge/api"
import { EthersEthereumProvider } from "@snowbridge/provider-ethers"
import { cryptoWaitReady } from "@polkadot/util-crypto"
import { formatUnits, Wallet } from "ethers"
import { bridgeInfoFor } from "@snowbridge/registry"

export const transferFromKusamaToEthereum = async (symbol: string, amount: bigint) => {
    await cryptoWaitReady()

    let env = "local_e2e"
    if (process.env.NODE_ENV !== undefined) {
        env = process.env.NODE_ENV
    }
    console.log(`Using environment '${env}'`)

    const info = bridgeInfoFor(env)
    const { registry } = info

    if (!registry.kusama) {
        throw Error("Kusama config is not set in the registry.")
    }

    const api = createApi({ info, ethereumProvider: new EthersEthereumProvider() })
    const context = api.context

    const polkadot_keyring = new Keyring({ type: "sr25519" })

    const ETHEREUM_ACCOUNT = new Wallet(
        process.env.ETHEREUM_KEY ?? "Your Key Goes Here",
        context.ethereum(),
    )
    const ETHEREUM_ACCOUNT_PUBLIC =
        process.env.ETHEREUM_ACCOUNT_PUBLIC ?? (await ETHEREUM_ACCOUNT.getAddress())
    const KUSAMA_ACCOUNT = polkadot_keyring.addFromUri(
        process.env.KUSAMA_SUBSTRATE_KEY ?? process.env.SUBSTRATE_KEY ?? "//Ferdie",
    )
    const KUSAMA_ACCOUNT_PUBLIC = KUSAMA_ACCOUNT.address

    console.log("eth", ETHEREUM_ACCOUNT_PUBLIC, "kusama", KUSAMA_ACCOUNT_PUBLIC)

    // Find the token by symbol from either Ethereum or Kusama AH assets
    const ethAssets = registry.ethereumChains[`ethereum_${registry.ethChainId}`].assets
    let TOKEN_ADDRESS = Object.keys(ethAssets)
        .map((t) => ethAssets[t])
        .find((asset) => asset.symbol.toLowerCase().startsWith(symbol.toLowerCase()))?.token

    if (!TOKEN_ADDRESS) {
        // Try Kusama AH assets
        const kusamaAHAssets =
            registry.kusama.parachains[`kusama_${registry.kusama.assetHubParaId}`]?.assets
        if (kusamaAHAssets) {
            for (const [token, asset] of Object.entries(kusamaAHAssets)) {
                if (asset.symbol.toLowerCase().startsWith(symbol.toLowerCase())) {
                    TOKEN_ADDRESS = token
                    break
                }
            }
        }
    }

    if (!TOKEN_ADDRESS) {
        throw Error(`No token found for ${symbol}`)
    }

    console.log("TOKEN_ADDRESS", TOKEN_ADDRESS)

    console.log("Kusama AssetHub to Ethereum")
    {
        // Step 0. Create a transfer implementation
        const transferImpl = api.sender(
            { kind: "kusama", id: registry.kusama.assetHubParaId },
            { kind: "ethereum", id: registry.ethChainId },
        )

        // Step 1. Get the delivery fee for the transaction
        let fee = await transferImpl.fee(TOKEN_ADDRESS)
        console.log("fee:", fee)

        // Step 2. Create a transfer tx
        const transfer = await transferImpl.tx(
            KUSAMA_ACCOUNT_PUBLIC,
            ETHEREUM_ACCOUNT_PUBLIC,
            TOKEN_ADDRESS,
            amount,
            fee,
        )

        // Step 3. Estimate the cost of the execution cost of the transaction
        console.log("call:", transfer.tx.inner.toHex())
        try {
            const feePayment = (
                await transfer.tx.paymentInfo(KUSAMA_ACCOUNT, { withSignedTransaction: true })
            ).toPrimitive() as any
            console.log("execution fee (KSM):", formatUnits(feePayment.partialFee, 12))
        } catch (e) {
            console.warn("Could not estimate execution fee:", e)
        }
        console.log("total delivery fee (KSM):", formatUnits(fee.totalFeeInKSM, 12))

        // Step 4. Validate the transaction.
        const validation = await transferImpl.validate(transfer)
        console.log("validation result", validation)

        // Step 5. Check validation logs for errors
        if (!validation.success) {
            throw Error(`validation has one of more errors.`)
        }
        if (process.env["DRY_RUN"] != "true") {
            // Step 6. Submit transaction and get receipt for tracking
            const response = await transferImpl.signAndSend(transfer, KUSAMA_ACCOUNT, {
                withSignedTransaction: true,
            })
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
