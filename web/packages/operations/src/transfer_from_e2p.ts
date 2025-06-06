import "dotenv/config"
import { transferToPolkadot } from "./transfer_to_polkadot"

const transfer = async (destinationChainId: number, symbol: string, amount: bigint) => {
    await transferToPolkadot(destinationChainId, symbol, amount)
}

if (process.argv.length != 5) {
    console.error("Expected arguments: `destinationChainId, symbol, amount`")
    process.exit(1)
}

transfer(parseInt(process.argv[2]), process.argv[3], BigInt(process.argv[4]))
    .then(() => process.exit(0))
    .catch((error) => {
        console.error("Error:", error)
        process.exit(1)
    })
