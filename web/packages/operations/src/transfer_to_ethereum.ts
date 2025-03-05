import "dotenv/config"
import { Keyring } from "@polkadot/keyring"
import { Context, environment, toEthereum } from "@snowbridge/api"
import { AbstractProvider, Wallet } from "ethers"
import cron from "node-cron"
import { cryptoWaitReady } from "@polkadot/util-crypto"

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

    const ETHEREUM_ACCOUNT = new Wallet(
        process.env["ETHEREUM_KEY"] || "your_key_here",
        context.ethereum()
    )
    const ETHEREUM_ACCOUNT_PUBLIC = await ETHEREUM_ACCOUNT.getAddress()
    const POLKADOT_ACCOUNT = process.env["SUBSTRATE_KEY"]
        ? polkadot_keyring.addFromUri(process.env["SUBSTRATE_KEY"])
        : polkadot_keyring.addFromUri("//Ferdie")

    const amount = 2_000_000_000_000n

    const WETH_CONTRACT = snowbridgeEnv.locations[0].erc20tokensReceivable.find(
        (t) => t.id === "WETH"
    )!.address

    console.log("# Asset Hub to Ethereum")
    {
        const plan = await toEthereum.validateSend(
            context,
            POLKADOT_ACCOUNT,
            1000,
            ETHEREUM_ACCOUNT_PUBLIC,
            WETH_CONTRACT,
            amount
        )
        console.log("Plan:", plan, plan.failure?.errors)
        const result = await toEthereum.send(context, POLKADOT_ACCOUNT, plan)
        console.log("Execute:", result)
    }
    await context.destroyContext()
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
