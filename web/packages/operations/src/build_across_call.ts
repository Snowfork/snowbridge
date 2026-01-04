import "dotenv/config"
import { Context, contextConfigFor } from "@snowbridge/api"
import { cryptoWaitReady } from "@polkadot/util-crypto"
import { SnowbridgeL1Adaptor, SnowbridgeL1Adaptor__factory } from "@snowbridge/contract-types"
import { Wallet } from "ethers"

const run = async (
    inputToken: string,
    outputToken: string,
    inputAmount: bigint,
    outputAmount: bigint,
    destinationChainId: bigint,
) => {
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
        context.ethereum(),
    )
    const ETHEREUM_ACCOUNT_PUBLIC = await ETHEREUM_ACCOUNT.getAddress()

    let l1AdaptorAddress =
        process.env["L1_ADAPTOR_ADDRESS"] ?? "0xb3D06e33Cc77c03968aeFECDeD91B5236BDa1983" // replace with actual address
    const l1Adaptor: SnowbridgeL1Adaptor = SnowbridgeL1Adaptor__factory.connect(
        l1AdaptorAddress,
        context.ethereum(),
    )
    let call = await l1Adaptor.interface.encodeFunctionData("depositToken", [
        {
            inputToken: inputToken,
            outputToken: outputToken,
            inputAmount: inputAmount,
            outputAmount: outputAmount,
            destinationChainId: destinationChainId,
        },
        ETHEREUM_ACCOUNT_PUBLIC,
    ])
    console.log("Calldata for SnowbridgeL1Adaptor:")
    console.log(call)
}

if (process.argv.length != 7) {
    console.error(
        "Expected arguments: `inputToken, outputToken, inputAmount, outputAmount, destinationChainId`",
    )
    process.exit(1)
}

run(
    process.argv[2],
    process.argv[3],
    BigInt(process.argv[4]),
    BigInt(process.argv[5]),
    BigInt(process.argv[6]),
)
    .then(() => process.exit(0))
    .catch((error) => {
        console.error("Error:", error)
        process.exit(1)
    })
