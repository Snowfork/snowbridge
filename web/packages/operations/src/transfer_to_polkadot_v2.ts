import { Keyring } from "@polkadot/keyring"
import { Context, toPolkadotSnowbridgeV2, contextConfigFor, toPolkadotV2 } from "@snowbridge/api"
import { cryptoWaitReady } from "@polkadot/util-crypto"
import { Wallet } from "ethers"
import { assetRegistryFor } from "@snowbridge/registry"

export const transferToPolkadot = async (destParaId: number, symbol: string, amount: bigint) => {
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
        context.ethereum()
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

    console.log("Ethereum to Polkadot")
    {
        // Step 0. Create a transfer implementation
        const transferImpl = toPolkadotSnowbridgeV2.createTransferImplementation(
            destParaId,
            registry,
            TOKEN_CONTRACT
        )
        // Step 1. Get the delivery fee for the transaction
        let fee = await transferImpl.getDeliveryFee(
            context,
            registry,
            TOKEN_CONTRACT,
            registry.assetHubParaId
        )

        console.log("fee: ", fee)
        // Step 2. Create a transfer tx
        const transfer = await transferImpl.createTransfer(
            await context.assetHub(),
            registry,
            destParaId,
            ETHEREUM_ACCOUNT_PUBLIC,
            POLKADOT_ACCOUNT_PUBLIC,
            TOKEN_CONTRACT,
            amount,
            fee
        )

        // Step 3. Validate the transaction.
        const validation = await transferImpl.validateTransfer(
            {
                ethereum: context.ethereum(),
                gateway: context.gatewayV2(),
                bridgeHub: await context.bridgeHub(),
                assetHub: await context.assetHub(),
                destParachain:
                    destParaId !== 1000 ? await context.parachain(destParaId) : undefined,
            },
            transfer
        )
        console.log("validation result", validation)

        // Step 4. Check validation logs for errors
        if (validation.logs.find((l) => l.kind == toPolkadotV2.ValidationKind.Error)) {
            throw Error(`validation has one of more errors.`)
        }
    }
    await context.destroyContext()
}
