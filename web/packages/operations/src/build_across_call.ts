import "dotenv/config"
import { Context, contextConfigFor } from "@snowbridge/api"
import { cryptoWaitReady } from "@polkadot/util-crypto"
import { SnowbridgeL1Adaptor, SnowbridgeL1Adaptor__factory } from "@snowbridge/contract-types"

const run = async () => {
    await cryptoWaitReady()
    let env = "local_e2e"
    if (process.env.NODE_ENV !== undefined) {
        env = process.env.NODE_ENV
    }
    console.log(`Using environment '${env}'`)

    const context = new Context(contextConfigFor(env))

    let l1AdaptorAddress = "0x13Ee7C49647b7B029ebCC63344b9098484E9Fb6c" // replace with actual address
    const l1Adaptor: SnowbridgeL1Adaptor = SnowbridgeL1Adaptor__factory.connect(
        l1AdaptorAddress,
        context.ethereum(),
    )
    let call = await l1Adaptor.interface.encodeFunctionData("swapToken", [
        {
            inputToken: "0xfFf9976782d46CC05630D1f6eBAb18b2324d6B14",
            outputToken: "0x4200000000000000000000000000000000000006",
            inputAmount: 1100000000000000n,
            outputAmount: 1000000000000000n,
            destinationChainId: 84532n,
        },
        "0x302f0b71b8ad3cf6dd90adb668e49b2168d652fd",
    ])
    console.log("Calldata for SnowbridgeL1Adaptor:")
    console.log(call)
}

run()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error("Error:", error)
        process.exit(1)
    })
