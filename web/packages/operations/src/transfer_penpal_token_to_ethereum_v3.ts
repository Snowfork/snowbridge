import "dotenv/config"
import { transferToEthereum } from "./transfer_to_ethereum_v3"

const transfer = async () => {
    await transferToEthereum(2000, "pal", 1_000_000_000n)
}

transfer()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error("Error:", error)
        process.exit(1)
    })
