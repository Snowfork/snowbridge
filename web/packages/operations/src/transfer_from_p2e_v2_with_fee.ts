import "dotenv/config"
import { transferToEthereum } from "./transfer_to_ethereum_v2"
import { assetsV2 } from "@snowbridge/api"
import { parachainLocation } from "@snowbridge/api/dist/xcmBuilder"

const transfer = async (sourceParaId: number, symbol: string, feeType: number, amount: bigint) => {
    let feeTokenLocation: any
    // Currently, we only support two types of fees to maintain V1 behavior
    if (feeType == 0) {
        feeTokenLocation = assetsV2.DOT_LOCATION
    } else {
        feeTokenLocation = parachainLocation(sourceParaId)
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
