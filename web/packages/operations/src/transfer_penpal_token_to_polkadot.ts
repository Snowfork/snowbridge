import "dotenv/config"
import { transferToPolkadot } from "./transfer_to_polkadot"

const transfer = async () => {
    await transferToPolkadot(2000, "pal", 100_000_000n)
}

transfer()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error("Error:", error)
        process.exit(1)
    })
