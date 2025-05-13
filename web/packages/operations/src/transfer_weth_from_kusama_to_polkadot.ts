import "dotenv/config"
import {Keyring} from "@polkadot/keyring"
import {Context, environment, toKusama} from "@snowbridge/api"
import {AbstractProvider} from "ethers"
import cron from "node-cron"
import {cryptoWaitReady} from "@polkadot/util-crypto"
import {fetchRegistry} from "./registry";
import {Direction} from "@snowbridge/api/dist/toKusama";

const transfer = async () => {
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

    const parachains: { [paraId: string]: string } = {}
    parachains[config.BRIDGE_HUB_PARAID.toString()] =
        process.env["BRIDGE_HUB_URL"] ?? config.PARACHAINS[config.BRIDGE_HUB_PARAID.toString()]
    parachains[config.ASSET_HUB_PARAID.toString()] =
        process.env["ASSET_HUB_URL"] ?? config.PARACHAINS[config.ASSET_HUB_PARAID.toString()]

    if (!kusamaConfig) {
        throw Error(`Kusama config should be set`)
    }

    const kusamaParachains: { [paraId: string]: string } = {}
    kusamaParachains[kusamaConfig?.BRIDGE_HUB_PARAID.toString()] = kusamaConfig?.PARACHAINS[config.BRIDGE_HUB_PARAID.toString()]
    kusamaParachains[kusamaConfig?.ASSET_HUB_PARAID.toString()] = kusamaConfig?.PARACHAINS[config.ASSET_HUB_PARAID.toString()]

    const ethChains: { [ethChainId: string]: string | AbstractProvider } = {}
    Object.keys(config.ETHEREUM_CHAINS)
        .forEach(ethChainId => ethChains[ethChainId.toString()] = config.ETHEREUM_CHAINS[ethChainId](process.env.REACT_APP_INFURA_KEY || ""))
    if (process.env["EXECUTION_NODE_URL"]) { ethChains[ethChainId.toString()] = process.env["EXECUTION_NODE_URL"] }

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
            parachains: parachains,
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

    const [polkadotAssetHub, polkadotBridgeHub, kusamaAssetHub, kusamaBridgeHub] = await Promise.all([
        context.assetHub(),
        context.bridgeHub(),
        context.kusamaAssetHub(),
        context.kusamaBridgeHub(),
    ])

    if (!kusamaAssetHub || !kusamaBridgeHub) {
        throw Error(`Kusama asset hub or bridge hub could not connect`)
    }

    const polkadot_keyring = new Keyring({ type: "sr25519" })

    const SUBSTRATE_ACCOUNT = process.env["SUBSTRATE_KEY"]
        ? polkadot_keyring.addFromUri(process.env["SUBSTRATE_KEY"])
        : polkadot_keyring.addFromUri("//Ferdie")

    const amount = 200000000000000n

    const registry = await fetchRegistry(env, context)

    const WETH_CONTRACT = snowbridgeEnv.locations[0].erc20tokensReceivable.find(
        (t) => t.id === "WETH"
    )!.address
    let sourceAccountHex = "0x460411e07f93dc4bc2b3a6cb67dad89ca26e8a54054d13916f74c982595c2e0e";
    let beneficiaryAccountHex = "0x460411e07f93dc4bc2b3a6cb67dad89ca26e8a54054d13916f74c982595c2e0e";

    const defaultBridgingFee = 333794429n;
    const direction = Direction.ToPolkadot;

    console.log("# Asset Hub Kusama to Asset Hub Polkadot")
    {
        // Step 1. Get the delivery fee for the transaction
        const fee = await toKusama.getDeliveryFee(
            kusamaAssetHub,
            direction,
            registry,
            defaultBridgingFee,
        )

        // Step 2. Create a transfer tx
        const transfer = await toKusama.createTransfer(
            kusamaAssetHub,
            direction,
            registry,
            sourceAccountHex,
            beneficiaryAccountHex,
            WETH_CONTRACT,
            amount,
            fee
        )

        // Step 3. Validate
        const validation = await toKusama.validateTransfer(
            {sourceAssetHub: kusamaAssetHub, destAssetHub: polkadotAssetHub, sourceBridgeHub: kusamaBridgeHub, destinationBridgeHub: polkadotBridgeHub},
            direction,
            transfer,
        );

        // Step 4. Check validation logs for errors
        if (validation.logs.find((l) => l.kind == toKusama.ValidationKind.Error)) {
            console.error("validation errors", validation.logs)
            throw Error(`validation has one of more errors.`)
        }

        // Step 5. Submit transaction and get receipt for tracking
        //const response = await toKusama.signAndSend(
        //    kusamaAssetHub,
        //    transfer,
        //    SUBSTRATE_ACCOUNT,
        //    { withSignedTransaction: true }
        //)
        //if (!response) {
        //    throw Error(`Transaction ${response} not included.`)
        //}
        //console.log("Success message", response.messageId)

        await context.destroyContext()
    }


}

if (process.argv.length != 3) {
    console.error("Expected one argument with Enum from `start|cron`")
    process.exit(1)
}

if (process.argv[2] == "start") {
    transfer()
        .then(() => process.exit(0))
        .catch((error) => {
            console.error("Error:", error)
            process.exit(1)
        })
} else if (process.argv[2] == "cron") {
    console.log("running cronjob")
    cron.schedule(process.env["CRON_EXPRESSION"] || "0 0 * * *", transfer)
}
