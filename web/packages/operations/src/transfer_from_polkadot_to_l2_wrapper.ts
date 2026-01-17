import "dotenv/config"
import { transferToEthereumL2 } from "./transfer_to_ethereum_l2"

const l2Transfer = async (
    sourceParaId: number,
    l2ChainId: number,
    symbol: string,
    amount: bigint,
) => {
    await transferToEthereumL2(sourceParaId, l2ChainId, symbol, amount)
}

if (process.argv.length != 6) {
    console.error("Expected arguments: `sourceParaId, l2ChainId, symbol, amount`")
    process.exit(1)
}

l2Transfer(
    parseInt(process.argv[2]),
    parseInt(process.argv[3]),
    process.argv[4],
    BigInt(process.argv[5]),
)
    .then(() => process.exit(0))
    .catch((error) => {
        console.error("Error:", error)
        process.exit(1)
    })
