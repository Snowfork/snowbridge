import { Keyring } from "@polkadot/keyring"
import { Context, contextConfigFor, environment } from "@snowbridge/api"
import { IGatewayV1__factory as IGateway__factory } from "@snowbridge/contract-types"
import { AbstractProvider, Contract, ethers, LogDescription, Wallet } from "ethers"
import { cryptoWaitReady } from "@polkadot/util-crypto"

export const registerERC20 = async (tokenAddress: string) => {
    await cryptoWaitReady()

    let env = "local_e2e"
    if (process.env.NODE_ENV !== undefined) {
        env = process.env.NODE_ENV
    }
    console.log(`Using environment '${env}'`)

    const context = new Context(contextConfigFor(env))

    const ETHEREUM_ACCOUNT = new Wallet(
        process.env.ETHEREUM_KEY ??
            "0x5e002a1af63fd31f1c25258f3082dc889762664cb8f218d86da85dff8b07b342",
        context.ethereum()
    )
    const ETHEREUM_ACCOUNT_PUBLIC = await ETHEREUM_ACCOUNT.getAddress()

    const polkadot_keyring = new Keyring({ type: "sr25519" })
    const POLKADOT_ACCOUNT = polkadot_keyring.addFromUri(process.env.SUBSTRATE_KEY ?? "//Ferdie")
    const POLKADOT_ACCOUNT_PUBLIC = POLKADOT_ACCOUNT.address

    console.log("eth", ETHEREUM_ACCOUNT_PUBLIC, "sub", POLKADOT_ACCOUNT_PUBLIC)

    const TOKEN_CONTRACT = tokenAddress

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

if (process.argv.length != 3) {
    console.error("Expected arguments: `tokenAddress`")
    process.exit(1)
}

registerERC20(process.argv[2])
    .then(() => process.exit(0))
    .catch((error) => {
        console.error("Error:", error)
        process.exit(1)
    })
