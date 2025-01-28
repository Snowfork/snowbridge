import "dotenv/config"
import { Keyring } from "@polkadot/keyring"
import {
    contextFactory,
    destroyContext,
    environment,
    toPolkadot,
} from "@snowbridge/api"
import { WETH9__factory } from "@snowbridge/contract-types"
import { Wallet } from "ethers"
import cron from "node-cron"

const transfer = async () => {
    let env = "local_e2e"
    if (process.env.NODE_ENV !== undefined) {
        env = process.env.NODE_ENV
    }
    const snowbridgeEnv = environment.SNOWBRIDGE_ENV[env]
    if (snowbridgeEnv === undefined) {
        throw Error(`Unknown environment '${env}'`)
    }

    const { config } = snowbridgeEnv

    const context = await contextFactory({
        ethereum: {
            execution_url: process.env["EXECUTION_NODE_URL"] || config.ETHEREUM_API(process.env.REACT_APP_INFURA_KEY || ""),
            beacon_url: process.env["BEACON_NODE_URL"] || config.BEACON_HTTP_API,
        },
        polkadot: {
            url: {
                bridgeHub: process.env["BRIDGE_HUB_URL"] || config.BRIDGE_HUB_URL,
                assetHub: process.env["ASSET_HUB_URL"] || config.ASSET_HUB_URL,
                relaychain: process.env["RELAY_CHAIN_URL"] || config.RELAY_CHAIN_URL,
            },
        },
        appContracts: {
            gateway: config.GATEWAY_CONTRACT,
            beefy: config.BEEFY_CONTRACT,
        },
    })
    const polkadot_keyring = new Keyring({ type: "sr25519" })

    const ETHEREUM_ACCOUNT = new Wallet(
        process.env["ETHEREUM_KEY"] || "0x5e002a1af63fd31f1c25258f3082dc889762664cb8f218d86da85dff8b07b342",
        context.ethereum.api
    )
    const POLKADOT_ACCOUNT = process.env["SUBSTRATE_KEY"]?polkadot_keyring.addFromUri(process.env["SUBSTRATE_KEY"]):polkadot_keyring.addFromUri("//Ferdie")
    const POLKADOT_ACCOUNT_PUBLIC = POLKADOT_ACCOUNT.address

    const amount = 2_000_000_000_000n

    const WETH_CONTRACT = snowbridgeEnv.locations[0].erc20tokensReceivable.find(
        (t) => t.id === "WETH"
    )!.address

    console.log("# Deposit and Approve WETH")
    {
        const weth9 = WETH9__factory.connect(WETH_CONTRACT, ETHEREUM_ACCOUNT)
        const depositResult = await weth9.deposit({ value: amount })
        const depositReceipt = await depositResult.wait()

        const approveResult = await weth9.approve(config.GATEWAY_CONTRACT, amount)
        const approveReceipt = await approveResult.wait()

        console.log('deposit tx', depositReceipt?.hash, 'approve tx', approveReceipt?.hash)
    }

    console.log("# Ethereum to Asset Hub")
    {
        const plan = await toPolkadot.validateSend(
            context,
            ETHEREUM_ACCOUNT,
            POLKADOT_ACCOUNT_PUBLIC,
            WETH_CONTRACT,
            1000,
            amount,
            BigInt(0)
        )
        console.log("Plan:", plan, plan.failure?.errors)
        let result = await toPolkadot.send(context, ETHEREUM_ACCOUNT, plan)
        console.log("Execute:", result)
    }
    await destroyContext(context)
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
