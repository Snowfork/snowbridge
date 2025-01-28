import { Keyring } from "@polkadot/keyring"
import { Signer } from "@polkadot/types/types"
import {
    contextFactory,
    destroyContext,
    environment,
    toEthereum,
    toPolkadot,
} from "@snowbridge/api"
import { WETH9__factory } from "@snowbridge/contract-types"
import { Wallet } from "ethers"

const monitor = async () => {
    let env = "local_e2e"
    if (process.env.NODE_ENV !== undefined) {
        env = process.env.NODE_ENV
    }
    const snwobridgeEnv = environment.SNOWBRIDGE_ENV[env]
    if (snwobridgeEnv === undefined) {
        throw Error(`Unknown environment '${env}'`)
    }
    console.log(`Using environment '${env}'`)

    const { config } = snwobridgeEnv

    const context = await contextFactory({
        ethereum: {
            execution_url: config.ETHEREUM_API(process.env.REACT_APP_INFURA_KEY || ""),
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
        "0x5e002a1af63fd31f1c25258f3082dc889762664cb8f218d86da85dff8b07b342",
        context.ethereum.api
    )
    const ETHEREUM_ACCOUNT_PUBLIC = await ETHEREUM_ACCOUNT.getAddress()
    const POLKADOT_ACCOUNT = polkadot_keyring.addFromUri("//Ferdie")
    const POLKADOT_ACCOUNT_PUBLIC = POLKADOT_ACCOUNT.address

    const amount = 10n

    const POLL_INTERVAL_MS = 10_000
    const WETH_CONTRACT = snwobridgeEnv.locations[0].erc20tokensReceivable.find(
        (t) => t.id === "WETH"
    )!.address

    console.log("# Deposit and Approve WETH")
    {
        const weth9 = WETH9__factory.connect(WETH_CONTRACT, ETHEREUM_ACCOUNT)
        const depositResult = await weth9.deposit({ value: amount })
        const depositReceipt = await depositResult.wait()

        const approveResult = await weth9.approve(config.GATEWAY_CONTRACT, amount * 2n)
        const approveReceipt = await approveResult.wait()

        console.log('deposit tx', depositReceipt?.hash, 'approve tx', approveReceipt?.hash)
    }

    console.log("# Ethereum to Asset Hub")
    {
        const destinationChainId = 1000
        const destinationFeeInDOT = 0n
        const totalFee = await toPolkadot.getSendFee(context, WETH_CONTRACT, destinationChainId, destinationFeeInDOT)
        const { tx } = await toPolkadot.createTx(
            context.config.appContracts.gateway,
            ETHEREUM_ACCOUNT_PUBLIC,
            POLKADOT_ACCOUNT_PUBLIC,
            WETH_CONTRACT,
            destinationChainId,
            amount,
            totalFee,
            destinationFeeInDOT,
        );
        console.log('Plan tx:', tx)
        console.log('Plan gas:', await context.ethereum.api.estimateGas(tx))
        console.log('Plan dry run:', await context.ethereum.api.call(tx))

        const plan = await toPolkadot.validateSend(
            context,
            ETHEREUM_ACCOUNT,
            POLKADOT_ACCOUNT_PUBLIC,
            WETH_CONTRACT,
            destinationChainId,
            amount,
            destinationFeeInDOT
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
        const assetHubUnsigned = await toEthereum.createTx(
            context.polkadot.api.assetHub,
            (await context.ethereum.api.getNetwork()).chainId,
            POLKADOT_ACCOUNT_PUBLIC,
            ETHEREUM_ACCOUNT_PUBLIC,
            WETH_CONTRACT,
            amount
        );
        console.log('call: ', assetHubUnsigned.tx.inner.toHex())
        console.log('utx: ', assetHubUnsigned.tx.toHex())
        console.log('payment: ', (await assetHubUnsigned.tx.paymentInfo(POLKADOT_ACCOUNT)).toHuman())
        console.log('dryRun: ', (
            await assetHubUnsigned.tx.dryRun(
                POLKADOT_ACCOUNT,
                { withSignedTransaction: true }
            )
        ).toHuman()
        )

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

    console.log("# Ethereum to Penpal")
    {
        const destinationChainId = 2000
        const destinationFeeInDOT = 4_000_000_000n
        const totalFee = await toPolkadot.getSendFee(context, WETH_CONTRACT, destinationChainId, destinationFeeInDOT)
        const { tx } = await toPolkadot.createTx(
            context.config.appContracts.gateway,
            ETHEREUM_ACCOUNT_PUBLIC,
            POLKADOT_ACCOUNT_PUBLIC,
            WETH_CONTRACT,
            destinationChainId,
            amount,
            totalFee,
            destinationFeeInDOT,
        );
        console.log('Plan tx:', tx)
        console.log('Plan gas:', await context.ethereum.api.estimateGas(tx))
        console.log('Plan Dry run:', await context.ethereum.api.call(tx))

        const plan = await toPolkadot.validateSend(
            context,
            ETHEREUM_ACCOUNT,
            POLKADOT_ACCOUNT_PUBLIC,
            WETH_CONTRACT,
            destinationChainId,
            amount,
            destinationFeeInDOT
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
        const assetHubUnsigned = await toEthereum.createTx(
            context.polkadot.api.assetHub,
            (await context.ethereum.api.getNetwork()).chainId,
            POLKADOT_ACCOUNT_PUBLIC,
            ETHEREUM_ACCOUNT_PUBLIC,
            WETH_CONTRACT,
            amount
        );
        console.log('call: ', assetHubUnsigned.tx.inner.toHex())
        console.log('utx: ', assetHubUnsigned.tx.toHex())
        console.log('payment: ', (await assetHubUnsigned.tx.paymentInfo(POLKADOT_ACCOUNT)).toHuman())
        console.log('dryRun: ', (
            await assetHubUnsigned.tx.dryRun(
                POLKADOT_ACCOUNT,
                { withSignedTransaction: true }
            )
        ).toHuman()
        )

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

    await destroyContext(context)
}

monitor()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error("Error:", error)
        process.exit(1)
    })
