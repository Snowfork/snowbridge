import "dotenv/config"
import { transferToKusama } from "./transfer_to_kusama_v2"

const transfer = async (symbol: string, amount: bigint) => {
    await transferToKusama(symbol, amount)
}

if (process.argv.length != 4) {
    console.error("Expected arguments: `symbol, amount`")
    process.exit(1)
}

transfer(process.argv[2], BigInt(process.argv[3]))
    .then(() => process.exit(0))
    .catch((error) => {
        console.error("Error:", error)
        process.exit(1)
    })
