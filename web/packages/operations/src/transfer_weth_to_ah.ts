import "dotenv/config"
import { Context, environment } from "@snowbridge/api"
import { WETH9__factory } from "@snowbridge/contract-types"
import { Wallet } from "ethers"
import { cryptoWaitReady } from "@polkadot/util-crypto"
import { transferToPolkadot } from "./transfer_to_polkadot"
import { fetchRegistry } from "./registry"

const transfer = async () => {
    let env = "local_e2e"
    if (process.env.NODE_ENV !== undefined) {
        env = process.env.NODE_ENV
    }
    const snwobridgeEnv = environment.SNOWBRIDGE_ENV[env]
    if (snwobridgeEnv === undefined) {
        throw Error(`Unknown environment '${env}'`)
    }
    console.log(`Using environment '${env}'`)

    const { name, config, ethChainId } = snwobridgeEnv
    await cryptoWaitReady()

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
            beacon_url: config.BEACON_HTTP_API,
        },
        polkadot: {
            assetHubParaId: config.ASSET_HUB_PARAID,
            bridgeHubParaId: config.BRIDGE_HUB_PARAID,
            relaychain: config.RELAY_CHAIN_URL,
            parachains: config.PARACHAINS,
        },
        appContracts: {
            gateway: config.GATEWAY_CONTRACT,
            beefy: config.BEEFY_CONTRACT,
        },
    })

    const ETHEREUM_ACCOUNT = new Wallet(
        process.env.ETHEREUM_KEY ?? "your key goes here",
        context.ethereum()
    )

    const amount = 10_000_000_000_000n

    const registry = await fetchRegistry(env, context)

    const assets = registry.ethereumChains[registry.ethChainId].assets
    const TOKEN_CONTRACT = Object.keys(assets)
        .map((t) => assets[t])
        .find((asset) => asset.symbol === "WETH")?.token

    console.log("# Deposit and Approve WETH")
    {
        // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
        const weth9 = WETH9__factory.connect(TOKEN_CONTRACT!, ETHEREUM_ACCOUNT)
        const depositResult = await weth9.deposit({ value: amount })
        const depositReceipt = await depositResult.wait()

        const approveResult = await weth9.approve(config.GATEWAY_CONTRACT, amount)
        const approveReceipt = await approveResult.wait()

        console.log("deposit tx", depositReceipt?.hash, "approve tx", approveReceipt?.hash)
    }

    await transferToPolkadot(1000, "WETH", amount)
}

transfer()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error("Error:", error)
        process.exit(1)
    })
