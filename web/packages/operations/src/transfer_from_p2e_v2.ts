import "dotenv/config"
import { transferToEthereum } from "./transfer_to_ethereum_v2"
import { ConcreteToken } from "@snowbridge/api/dist/assets_v2"

const transfer = async (sourceParaId: number, tokens: ConcreteToken[]) => {
    await transferToEthereum(sourceParaId, tokens)
}

if (process.argv.length > 13 || process.argv.length < 5) {
    console.error("Expected arguments: `sourceParaId, (address, amount)...`")
    process.exit(1)
}

let tokenPairs = (process.argv.length - 3) / 2
let tokens: ConcreteToken[] = []
for (let i = 0; i < tokenPairs; i++) {
    const token = process.argv[3 + i * 2]
    const amount = BigInt(process.argv[4 + i * 2])
    tokens.push({ address: token, amount: amount })
}
transfer(parseInt(process.argv[2]), tokens)
    .then(() => process.exit(0))
    .catch((error) => {
        console.error("Error:", error)
        process.exit(1)
    })
