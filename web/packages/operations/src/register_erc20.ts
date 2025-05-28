import { Keyring } from "@polkadot/keyring"
import { Context, environment } from "@snowbridge/api"
import { IGatewayV1__factory as IGateway__factory } from "@snowbridge/contract-types"
import { AbstractProvider, Contract, ethers, LogDescription, Wallet } from "ethers"
import { cryptoWaitReady } from "@polkadot/util-crypto"

export const registerERC20 = async (symbol: string) => {
    let env = "local_e2e"
    if (process.env.NODE_ENV !== undefined) {
        env = process.env.NODE_ENV
    }
    const snwobridgeEnv = environment.SNOWBRIDGE_ENV[env]
    if (snwobridgeEnv === undefined) {
        throw Error(`Unknown environment '${env}'`)
    }
    console.log(`Using environment '${env}'`)

    const { config, ethChainId, name } = snwobridgeEnv
    await cryptoWaitReady()

    const ethApikey = process.env.REACT_APP_INFURA_KEY || ""
    const ethChains: { [ethChainId: string]: string | AbstractProvider } = {}
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
    const ETHEREUM_ACCOUNT_PUBLIC = await ETHEREUM_ACCOUNT.getAddress()

    const polkadot_keyring = new Keyring({ type: "sr25519" })
    const POLKADOT_ACCOUNT = polkadot_keyring.addFromUri(
        process.env.SUBSTRATE_KEY ?? "your key goes here"
    )
    const POLKADOT_ACCOUNT_PUBLIC = POLKADOT_ACCOUNT.address

    console.log("eth", ETHEREUM_ACCOUNT_PUBLIC, "sub", POLKADOT_ACCOUNT_PUBLIC)

    const TOKEN_CONTRACT = snwobridgeEnv.locations[0].erc20tokensReceivable.find((t) =>
        t.id.toLowerCase().startsWith(symbol.toLowerCase())
    )!.address

    const ifce = IGateway__factory.createInterface()
    const gateway = new Contract(context.config.appContracts.gateway, ifce)
    const tx = await gateway.getFunction("registerToken").populateTransaction(TOKEN_CONTRACT, {
        value: ethers.parseEther("1"),
        from: ETHEREUM_ACCOUNT_PUBLIC,
    })
    console.log("Plan tx:", tx)
    console.log("Plan gas:", await context.ethereum().estimateGas(tx))
    console.log("Plan dry run:", await context.ethereum().call(tx))

    const response = await ETHEREUM_ACCOUNT.sendTransaction(tx)
    let receipt = await response.wait(1)

    if (receipt === null) {
        throw new Error("Error waiting for transaction completion")
    }

    if (receipt?.status !== 1) {
        return {
            failure: {
                receipt: receipt,
            },
        }
    }
    const events: LogDescription[] = []
    receipt.logs.forEach((log) => {
        let event = gateway.interface.parseLog({
            topics: [...log.topics],
            data: log.data,
        })
        if (event !== null) {
            events.push(event)
        }
    })
    const messageAccepted = events.find((log) => log.name === "OutboundMessageAccepted")
    if (!messageAccepted) {
        throw Error(`Transaction ${receipt.hash} did not emit a message.`)
    }
    console.log("Success message", receipt.hash)

    context.destroyContext()
}
