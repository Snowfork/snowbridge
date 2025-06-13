import "dotenv/config"
import { transferToEthereum } from "./transfer_to_ethereum"

const transfer = async () => {
    await transferToEthereum(2313, "xrqcy", 1_000_000n)
}

transfer()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error("Error:", error)
        process.exit(1)
    })
