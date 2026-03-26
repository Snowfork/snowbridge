import "dotenv/config"
import { transferToEthereum } from "./transfer_to_ethereum_v2"
import { xcmBuilder } from "@snowbridge/api"

const transfer = async (sourceParaId: number, symbol: string, feeType: number, amount: bigint) => {
    let feeTokenLocation: any
    // Currently, we only support two types of fees to maintain V1 behavior
    if (feeType == 0) {
        feeTokenLocation = xcmBuilder.DOT_LOCATION
    } else {
        feeTokenLocation = xcmBuilder.parachainLocation(sourceParaId)
    }
    await transferToEthereum(sourceParaId, symbol, amount, feeTokenLocation)
}

if (process.argv.length != 6) {
    console.error("Expected arguments: `destinationChainId, symbol, feeType, amount`")
    process.exit(1)
}

transfer(
    parseInt(process.argv[2]),
    process.argv[3],
    parseInt(process.argv[4]),
    BigInt(process.argv[5]),
)
    .then(() => process.exit(0))
    .catch((error) => {
        console.error("Error:", error)
        process.exit(1)
    })
