import "dotenv/config"
import { Keyring } from "@polkadot/keyring"
import { EthersEthereumProvider, createApi } from "@snowbridge/api"
import { Direction } from "@snowbridge/api/dist/forKusama"
import { bridgeInfoFor } from "@snowbridge/registry"

export const transferForKusama = async (
    transferName: string,
    direction: Direction,
    amount: bigint,
    tokenName: string,
) => {
    let env = "local_e2e"
    if (process.env.NODE_ENV !== undefined) {
        env = process.env.NODE_ENV
    }
    const info = bridgeInfoFor(env)
    const { registry, environment: snowbridgeEnv } = info
    if (snowbridgeEnv === undefined) {
        throw Error(`Unknown environment '${env}'`)
    }

    const api = createApi({ info, ethereumProvider: new EthersEthereumProvider() })
    const context = api.context

    const polkadot_keyring = new Keyring({ type: "sr25519" })

    const SOURCE_ACCOUNT = process.env["SOURCE_SUBSTRATE_KEY"]
        ? polkadot_keyring.addFromUri(process.env["SOURCE_SUBSTRATE_KEY"])
        : polkadot_keyring.addFromUri("//Ferdie")
    const DEST_ACCOUNT = process.env["DEST_SUBSTRATE_KEY"]
        ? polkadot_keyring.addFromUri(process.env["DEST_SUBSTRATE_KEY"])
        : polkadot_keyring.addFromUri("//Ferdie")

    const SOURCE_ACCOUNT_PUBLIC = SOURCE_ACCOUNT.address
    const DEST_ACCOUNT_PUBLIC = DEST_ACCOUNT.address

    let tokenAddress
    if (tokenName == "ETH") {
        tokenAddress = "0x0000000000000000000000000000000000000000"
    } else {
        // look for Ethereum assets
        const assets = registry.ethereumChains[`ethereum_${registry.ethChainId}`].assets
        for (const [token, asset] of Object.entries(assets)) {
            if (asset.symbol === tokenName) {
                tokenAddress = token
            }
        }
    }

    if (!tokenAddress) {
        // look for Parachain assets
        const assets = registry.parachains[`polkadot_${registry.assetHubParaId}`].assets
        for (const [token, asset] of Object.entries(assets)) {
            if (asset.symbol === tokenName) {
                tokenAddress = token
            }
        }
    }

    if (!tokenAddress) {
        throw Error(`Token ${tokenName} not found`)
    }

    console.log(transferName)
    {
        const transferImpl =
            direction == Direction.ToPolkadot
                ? api.transfer(
                      { kind: "kusama", id: registry.kusama!.assetHubParaId },
                      { kind: "polkadot", id: registry.assetHubParaId },
                  )
                : api.transfer(
                      { kind: "polkadot", id: registry.assetHubParaId },
                      { kind: "kusama", id: registry.kusama!.assetHubParaId },
                  )

        // Step 1. Get the delivery fee for the transaction
        const fee = await transferImpl.getDeliveryFee(tokenAddress)

        // Step 2. Create a transfer tx
        const transfer = await transferImpl.createTransfer(
            SOURCE_ACCOUNT_PUBLIC,
            DEST_ACCOUNT_PUBLIC,
            tokenAddress,
            amount,
            fee,
        )

        // Step 3. Validate
        const validation = await transferImpl.validateTransfer(transfer)

        // Step 4. Check validation logs for errors
        if (!validation.success) {
            console.error("validation errors", validation.logs)
            throw Error(`validation has one of more errors.`)
        }

        // Step 5. Submit transaction and get receipt for tracking
        const response = await transferImpl.signAndSend(transfer, SOURCE_ACCOUNT, {
            withSignedTransaction: true,
        })
        if (!response) {
            throw Error(`Transaction ${response} not included.`)
        }
        console.log("Success message", response.messageId)

        await context.destroyContext()
    }
}
