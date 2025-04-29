import "dotenv/config"
import { Keyring } from "@polkadot/keyring"
import {Context, environment, toEthereumV2, toKusama} from "@snowbridge/api"
import { AbstractProvider, Wallet } from "ethers"
import cron from "node-cron"
import { cryptoWaitReady } from "@polkadot/util-crypto"
import {ApiPromise} from "@polkadot/api";
import {fetchRegistry} from "./registry";

const transfer = async () => {
    let env = "local_e2e"
    if (process.env.NODE_ENV !== undefined) {
        env = process.env.NODE_ENV
    }
    const snowbridgeEnv = environment.SNOWBRIDGE_ENV[env]
    if (snowbridgeEnv === undefined) {
        throw Error(`Unknown environment '${env}'`)
    }

    const { config, ethChainId, name } = snowbridgeEnv
    await cryptoWaitReady()

    const parachains: { [paraId: string]: string } = {}
    parachains[config.BRIDGE_HUB_PARAID.toString()] =
        process.env["BRIDGE_HUB_URL"] ?? config.PARACHAINS[config.BRIDGE_HUB_PARAID.toString()]
    parachains[config.ASSET_HUB_PARAID.toString()] =
        process.env["ASSET_HUB_URL"] ?? config.PARACHAINS[config.ASSET_HUB_PARAID.toString()]

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
        appContracts: {
            gateway: config.GATEWAY_CONTRACT,
            beefy: config.BEEFY_CONTRACT,
        },
    })
    const polkadot_keyring = new Keyring({ type: "sr25519" })

    const POLKADOT_ACCOUNT = process.env["SUBSTRATE_KEY"]
        ? polkadot_keyring.addFromUri(process.env["SUBSTRATE_KEY"])
        : polkadot_keyring.addFromUri("//Ferdie")

    const amount = 200000000000000n

    const registry = await fetchRegistry(env, context)

    const WETH_CONTRACT = snowbridgeEnv.locations[0].erc20tokensReceivable.find(
        (t) => t.id === "WETH"
    )!.address
    let totalFeeInDot = 500000000n; // 1 DOT
    let sourceAccountHex = "0x460411e07f93dc4bc2b3a6cb67dad89ca26e8a54054d13916f74c982595c2e0e";

    const [assetHub] = await Promise.all([
        context.assetHub(),
    ])

    console.log("# Asset Hub Polkadot to Asset Hub Kusama")
    {
        // Step 1. Create a transfer tx
        const transfer = await toKusama.createTransfer(
            assetHub,
            registry,
            sourceAccountHex,
            sourceAccountHex,
            WETH_CONTRACT,
            amount,
            totalFeeInDot
        )

        // Step 2. Validate
        const validation = await toKusama.validateTransfer(
            assetHub,
            transfer,
        );
        console.log("validation result", validation)

        // Step 5. Check validation logs for errors
        if (validation.logs.find((l) => l.kind == toKusama.ValidationKind.Error)) {
            throw Error(`validation has one of more errors.`)
        }

        // Step 3. Submit transaction and get receipt for tracking
       // const response = await toKusama.signAndSend(
       //     assetHub,
       //     transfer,
       //     POLKADOT_ACCOUNT,
       //     { withSignedTransaction: true }
       // )
       // if (!response) {
       //     throw Error(`Transaction ${response} not included.`)
       // }
       // console.log("Success message", response.messageId)

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
