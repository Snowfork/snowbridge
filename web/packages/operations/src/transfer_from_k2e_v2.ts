import "dotenv/config"
import { transferFromKusamaToEthereum } from "./transfer_from_kusama_to_ethereum_v2"

const transfer = async (symbol: string, amount: bigint) => {
    await transferFromKusamaToEthereum(symbol, amount)
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
