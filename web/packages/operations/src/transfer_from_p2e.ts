import "dotenv/config"
import { transferToEthereum } from "./transfer_to_ethereum"

const transfer = async (sourceParaId: number, symbol: string, amount: bigint) => {
    await transferToEthereum(sourceParaId, symbol, amount)
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
