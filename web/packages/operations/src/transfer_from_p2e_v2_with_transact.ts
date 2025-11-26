import "dotenv/config"
import { transferToEthereum } from "./transfer_to_ethereum_v2"
import { ConcreteToken } from "@snowbridge/api/dist/assets_v2"

const transact = async (
    sourceParaId: number,
    target: string,
    calldata: string,
    value: bigint,
    gas: bigint,
    tokens: ConcreteToken[],
) => {
    await transferToEthereum(sourceParaId, tokens, {
        feeTokenLocation: undefined,
        contractCall: {
            target,
            calldata,
            value,
            gas,
        },
    })
}

let tokenPairs = (process.argv.length - 7) / 2
let tokens: ConcreteToken[] = []
for (let i = 0; i < tokenPairs; i++) {
    const token = process.argv[7 + i * 2]
    const amount = BigInt(process.argv[8 + i * 2])
    tokens.push({ address: token, amount: amount })
}

transact(
    parseInt(process.argv[2]),
    process.argv[3],
    process.argv[4],
    BigInt(process.argv[5]),
    BigInt(process.argv[6]),
    tokens,
)
    .then(() => process.exit(0))
    .catch((error) => {
        console.error("Error:", error)
        process.exit(1)
    })
