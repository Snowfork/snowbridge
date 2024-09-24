import "dotenv/config"
import { Keyring } from "@polkadot/keyring"
import {
    contextFactory,
    destroyContext,
    environment,
    toEthereum,
    toPolkadot,
} from "@snowbridge/api"
import { WETH9__factory } from "@snowbridge/contract-types"
import { Wallet } from "ethers"
import cron from "node-cron"

const monitor = async () => {
    let env = "local_e2e"
    if (process.env.NODE_ENV !== undefined) {
        env = process.env.NODE_ENV
    }
    const snwobridgeEnv = environment.SNOWBRIDGE_ENV[env]
    if (snwobridgeEnv === undefined) {
        throw Error(`Unknown environment '${env}'`)
    }

    const { config } = snwobridgeEnv

    const context = await contextFactory({
        ethereum: {
            execution_url: process.env["EXECUTION_NODE_URL"] || config.ETHEREUM_API(process.env.REACT_APP_INFURA_KEY || ""),
            beacon_url: config.BEACON_HTTP_API,
        },
        polkadot: {
            url: {
                bridgeHub: config.BRIDGE_HUB_URL,
                assetHub: config.ASSET_HUB_URL,
                relaychain: config.RELAY_CHAIN_URL,
                parachains: config.PARACHAINS,
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
    const ETHEREUM_ACCOUNT_PUBLIC = await ETHEREUM_ACCOUNT.getAddress()
    const POLKADOT_ACCOUNT = process.env["SUBSTRATE_KEY"]?polkadot_keyring.addFromUri(process.env["SUBSTRATE_KEY"]):polkadot_keyring.addFromUri("//Ferdie")
    const POLKADOT_ACCOUNT_PUBLIC = POLKADOT_ACCOUNT.address

    const amount = 2_000_000_000_000n

    const POLL_INTERVAL_MS = 10_000
    const WETH_CONTRACT = snwobridgeEnv.locations[0].erc20tokensReceivable.find(
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
        while (true) {
            const { status } = await toPolkadot.trackSendProgressPolling(context, result)
            if (status !== "pending") {
                break
            }
            await new Promise((r) => setTimeout(r, POLL_INTERVAL_MS))
        }
        console.log("Complete:", result)
    }

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
        while (true) {
            const { status } = await toEthereum.trackSendProgressPolling(context, result)
            if (status !== "pending") {
                break
            }
            await new Promise((r) => setTimeout(r, POLL_INTERVAL_MS))
        }
        console.log("Complete:", result)
    }
    // Disable penpal transfers
    if (process.env["PENPAL_TRANSFER"] == 'true') {
        console.log("# Ethereum to Penpal")
        {
            const plan = await toPolkadot.validateSend(
                context,
                ETHEREUM_ACCOUNT,
                POLKADOT_ACCOUNT_PUBLIC,
                WETH_CONTRACT,
                2000,
                amount,
                BigInt(4_000_000_000)
            )
            console.log("Plan:", plan, plan.failure?.errors)
            let result = await toPolkadot.send(context, ETHEREUM_ACCOUNT, plan)
            console.log("Execute:", result)
            while (true) {
                const { status } = await toPolkadot.trackSendProgressPolling(context, result)
                if (status !== "pending") {
                    break
                }
                await new Promise((r) => setTimeout(r, POLL_INTERVAL_MS))
            }
            console.log("Complete:", result)
        }

        console.log("# Penpal to Ethereum")
        {
            const plan = await toEthereum.validateSend(
                context,
                POLKADOT_ACCOUNT,
                2000,
                ETHEREUM_ACCOUNT_PUBLIC,
                WETH_CONTRACT,
                amount
            )
            console.log("Plan:", plan, plan.failure?.errors)
            const result = await toEthereum.send(context, POLKADOT_ACCOUNT, plan)
            console.log("Execute:", result)
            while (true) {
                const { status } = await toEthereum.trackSendProgressPolling(context, result)
                if (status !== "pending") {
                    break
                }
                await new Promise((r) => setTimeout(r, POLL_INTERVAL_MS))
            }
            console.log("Complete:", result)
        }
    }
    await destroyContext(context)
}

if (process.argv.length != 3) {
    console.error("Expected one argument with Enum from `start|cron|init`")
    process.exit(1)
}

if (process.argv[2] == "start") {
    monitor()
        .then(() => process.exit(0))
        .catch((error) => {
            console.error("Error:", error)
            process.exit(1)
        })
} else if (process.argv[2] == "cron") {
    console.log("running cronjob")
    cron.schedule(process.env["CRON_EXPRESSION"] || "0 0 * * *", monitor)
}
