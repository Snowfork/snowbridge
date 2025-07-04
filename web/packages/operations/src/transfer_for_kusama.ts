import "dotenv/config"
import { Keyring } from "@polkadot/keyring"
import { Context, environment, forKusama } from "@snowbridge/api"
import { AbstractProvider } from "ethers"
import cron from "node-cron"
import { cryptoWaitReady } from "@polkadot/util-crypto"
import { assetRegistryFor } from "@snowbridge/registry"
import { Direction } from "@snowbridge/api/dist/forKusama"

export const transferForKusama = async (
    transferName: string,
    direction: Direction,
    amount: bigint,
    tokenName: string
) => {
    let env = "local_e2e"
    if (process.env.NODE_ENV !== undefined) {
        env = process.env.NODE_ENV
    }
    const snowbridgeEnv = environment.SNOWBRIDGE_ENV[env]
    if (snowbridgeEnv === undefined) {
        throw Error(`Unknown environment '${env}'`)
    }

    const { config, kusamaConfig, ethChainId, name } = snowbridgeEnv
    await cryptoWaitReady()

    if (!kusamaConfig) {
        throw Error(`Kusama config should be set`)
    }

    const kusamaParachains: { [paraId: string]: string } = {}
    kusamaParachains[kusamaConfig?.BRIDGE_HUB_PARAID.toString()] =
        kusamaConfig?.PARACHAINS[config.BRIDGE_HUB_PARAID.toString()]
    kusamaParachains[kusamaConfig?.ASSET_HUB_PARAID.toString()] =
        kusamaConfig?.PARACHAINS[config.ASSET_HUB_PARAID.toString()]

    const ethApikey = process.env.REACT_APP_INFURA_KEY || ""
    const ethChains: { [ethChainId: string]: string } = {}
    Object.keys(config.ETHEREUM_CHAINS).forEach(
        (ethChainId) =>
            (ethChains[ethChainId.toString()] = config.ETHEREUM_CHAINS[ethChainId](ethApikey))
    )
    const context = new Context({
        environment: name,
        ethereum: {
            ethChainId,
            ethChains,
            beacon_url: process.env["BEACON_NODE_URL"] || config.BEACON_HTTP_API,
        },
        polkadot: {
            assetHubParaId: config.ASSET_HUB_PARAID,
            bridgeHubParaId: config.BRIDGE_HUB_PARAID,
            parachains: config.PARACHAINS,
            relaychain: process.env["RELAY_CHAIN_URL"] || config.RELAY_CHAIN_URL,
        },
        kusama: {
            assetHubParaId: kusamaConfig.ASSET_HUB_PARAID,
            bridgeHubParaId: kusamaConfig.BRIDGE_HUB_PARAID,
            parachains: kusamaParachains,
        },
        appContracts: {
            gateway: config.GATEWAY_CONTRACT,
            beefy: config.BEEFY_CONTRACT,
        },
    })

    const [polkadotAssetHub, kusamaAssetHub] = await Promise.all([
        context.assetHub(),
        context.kusamaAssetHub(),
    ])

    if (!kusamaAssetHub) {
        throw Error(`Kusama asset hub could not connect`)
    }

    const polkadot_keyring = new Keyring({ type: "sr25519" })

    const SOURCE_ACCOUNT = process.env["SOURCE_SUBSTRATE_KEY"]
        ? polkadot_keyring.addFromUri(process.env["SOURCE_SUBSTRATE_KEY"])
        : polkadot_keyring.addFromUri("//Ferdie")
    const DEST_ACCOUNT = process.env["DEST_SUBSTRATE_KEY"]
        ? polkadot_keyring.addFromUri(process.env["DEST_SUBSTRATE_KEY"])
        : polkadot_keyring.addFromUri("//Ferdie")

    const registry = assetRegistryFor(env)

    const SOURCE_ACCOUNT_PUBLIC = SOURCE_ACCOUNT.address
    const DEST_ACCOUNT_PUBLIC = DEST_ACCOUNT.address

    let sourceAssetHub
    let destAssetHub
    if (direction == Direction.ToPolkadot) {
        sourceAssetHub = kusamaAssetHub
        destAssetHub = polkadotAssetHub
    } else {
        sourceAssetHub = polkadotAssetHub
        destAssetHub = kusamaAssetHub
    }

    let tokenAddress
    if (tokenName == "DOT" || tokenName == "KSM") {
        const assets = registry.parachains[registry.assetHubParaId].assets
        for (const [token, asset] of Object.entries(assets)) {
            if (asset.symbol === tokenName) {
                tokenAddress = token
            }
        }
    } else if (tokenName == "ETH") {
        tokenAddress = "0x0000000000000000000000000000000000000000"
    } else {
        tokenAddress = snowbridgeEnv.locations[0].erc20tokensReceivable.find(
            (t) => t.id === tokenName
        )!.address
    }

    if (!tokenAddress) {
        throw Error(`Token ${tokenName} not found`)
    }

    console.log(transferName)
    {
        // Step 1. Get the delivery fee for the transaction
        const fee = await forKusama.getDeliveryFee(
            sourceAssetHub,
            destAssetHub,
            direction,
            registry,
            tokenAddress
        )

        // Step 2. Create a transfer tx
        const transfer = await forKusama.createTransfer(
            sourceAssetHub,
            direction,
            registry,
            SOURCE_ACCOUNT_PUBLIC,
            DEST_ACCOUNT_PUBLIC,
            tokenAddress,
            amount,
            fee
        )

        // Step 3. Validate
        const validation = await forKusama.validateTransfer(
            { sourceAssetHub, destAssetHub },
            direction,
            transfer
        )

        // Step 4. Check validation logs for errors
        if (validation.logs.find((l) => l.kind == forKusama.ValidationKind.Error)) {
            console.error("validation errors", validation.logs)
            throw Error(`validation has one of more errors.`)
        }

        // Step 5. Submit transaction and get receipt for tracking
        const response = await forKusama.signAndSend(sourceAssetHub, transfer, SOURCE_ACCOUNT, {
            withSignedTransaction: true,
        })
        if (!response) {
            throw Error(`Transaction ${response} not included.`)
        }
        console.log("Success message", response.messageId)

        await context.destroyContext()
    }
}
