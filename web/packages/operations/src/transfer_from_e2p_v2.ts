import "dotenv/config"
import { transferToPolkadot } from "./transfer_to_polkadot_v2"

const transfer = async (destParaId: number, symbol: string, amount: bigint, feeAsset?: string) => {
    await transferToPolkadot(destParaId, symbol, amount, feeAsset)
}

if (process.argv.length < 5 || process.argv.length > 6) {
    console.error("Expected arguments: `destinationChainId, symbol, amount, [feeAsset]`")
    process.exit(1)
}

transfer(parseInt(process.argv[2]), process.argv[3], BigInt(process.argv[4]), process.argv[5])
    .then(() => process.exit(0))
    .catch((error) => {
        console.error("Error:", error)
        process.exit(1)
    })
