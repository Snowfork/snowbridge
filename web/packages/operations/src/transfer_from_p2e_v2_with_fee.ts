import "dotenv/config"
import { transferToEthereum } from "./transfer_to_ethereum_v2"
import { xcmBuilder } from "@snowbridge/api"
import { ConcreteToken } from "@snowbridge/api/dist/assets_v2"

const transfer = async (sourceParaId: number, feeType: number, tokens: ConcreteToken[]) => {
    let feeTokenLocation: any
    // Currently, we only support two types of fees to maintain V1 behavior
    if (feeType == 0) {
        feeTokenLocation = xcmBuilder.DOT_LOCATION
    } else {
        feeTokenLocation = xcmBuilder.parachainLocation(sourceParaId)
    }
    await transferToEthereum(sourceParaId, tokens, feeTokenLocation)
}

let tokenPairs = (process.argv.length - 4) / 2
let tokens: ConcreteToken[] = []
for (let i = 0; i < tokenPairs; i++) {
    const token = process.argv[4 + i * 2]
    const amount = BigInt(process.argv[5 + i * 2])
    tokens.push({ address: token, amount: amount })
}
transfer(parseInt(process.argv[2]), parseInt(process.argv[3]), tokens)
    .then(() => process.exit(0))
    .catch((error) => {
        console.error("Error:", error)
        process.exit(1)
    })
