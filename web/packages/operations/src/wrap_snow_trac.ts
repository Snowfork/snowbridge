import { Keyring } from "@polkadot/keyring"
import { Context } from "@snowbridge/api"
import { cryptoWaitReady } from "@polkadot/util-crypto"
import { bridgeInfoFor } from "@snowbridge/registry"
import { NeurowebParachain } from "@snowbridge/api/dist/parachains/neuroweb"

const wrapSnowTRAC = async () => {
    await cryptoWaitReady()

    let env = "local_e2e"
    if (process.env.NODE_ENV !== undefined) {
        env = process.env.NODE_ENV
    }
    console.log(`Using environment '${env}'`)

    const { registry, environment } = bridgeInfoFor(env)
    const context = new Context(environment)

    const polkadot_keyring = new Keyring({ type: "sr25519" })

    const POLKADOT_ACCOUNT = polkadot_keyring.addFromUri(process.env.SUBSTRATE_KEY ?? "//Ferdie")
    const POLKADOT_ACCOUNT_PUBLIC = POLKADOT_ACCOUNT.address

    console.log("sub", POLKADOT_ACCOUNT_PUBLIC)

    const neuroWebParaId = 2043

    if (!registry.parachains[neuroWebParaId]) {
        throw Error("Neuroweb parachain config not set in registry")
    }
    const parachainInfo = registry.parachains[neuroWebParaId].info

    console.log("Wrap SnowTRAC to TRAC")
    {
        const parachain = await context.parachain(neuroWebParaId)
        const neuroWeb = new NeurowebParachain(
            parachain,
            neuroWebParaId,
            parachainInfo.specName,
            parachainInfo.specVersion,
        )

        const fee = await neuroWeb.wrapExecutionFeeInNative(parachain)
        console.log("Execution fee:", fee)

        const balance = await neuroWeb.snowTRACBalance(POLKADOT_ACCOUNT_PUBLIC, registry.ethChainId)
        console.log("SnowTRAC balance:", balance)

        if (balance == 0n) {
            console.error("SnowTRAC balance is 0, nothing to wrap")
            process.exit(1)
        }

        const wrapTx = neuroWeb.createWrapTx(balance)

        await wrapTx.signAndSend(POLKADOT_ACCOUNT, { nonce: -1 }, (result) => {
            console.log(`Transaction status: ${result.status}`)
            if (result.status.isInBlock) {
                console.log(`Transaction included in block: ${result.status.asInBlock}`)
            } else if (result.status.isFinalized) {
                console.log(`Transaction finalized: ${result.status.asFinalized}`)
                process.exit(0)
            }
        })
    }
}

if (process.argv.length != 2) {
    console.error("Invalid arguments")
    process.exit(1)
}

wrapSnowTRAC()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error("Error:", error)
        process.exit(1)
    })
