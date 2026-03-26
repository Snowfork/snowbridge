import "dotenv/config"
import { transferToEthereum } from "./transfer_to_ethereum_v2"

const transact = async (
    sourceParaId: number,
    symbol: string,
    amount: bigint,
    target: string,
    calldata: string,
    value: bigint,
    gas: bigint,
) => {
    await transferToEthereum(sourceParaId, symbol, amount, {
        feeTokenLocation: undefined,
        contractCall: {
            target,
            calldata,
            value,
            gas,
        },
    })
}

if (process.argv.length != 9) {
    console.error(
        "Expected arguments: `destinationChainId, symbol, amount, target, calldata, value, gas`",
    )
    process.exit(1)
}

transact(
    parseInt(process.argv[2]),
    process.argv[3],
    BigInt(process.argv[4]),
    process.argv[5],
    process.argv[6],
    BigInt(process.argv[7]),
    BigInt(process.argv[8]),
)
    .then(() => process.exit(0))
    .catch((error) => {
        console.error("Error:", error)
        process.exit(1)
    })
