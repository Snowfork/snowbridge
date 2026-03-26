import "dotenv/config"
import { transferToPolkadot } from "./transfer_from_l2_to_polkadot"

const transfer = async (l2ChainId: number, destParaId: number, symbol: string, amount: bigint) => {
    await transferToPolkadot(l2ChainId, destParaId, symbol, amount)
}

if (process.argv.length != 6) {
    console.error("Expected arguments: `l2ChainId, destinationChainId, symbol, amount`")
    process.exit(1)
}

transfer(
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
