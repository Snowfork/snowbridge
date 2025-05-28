import "dotenv/config"
import cron from "node-cron"
import { transferToEthereum } from "./transfer_to_ethereum_v3"

const transfer = async () => {
    await transferToEthereum(1000, "Roc", 2_000_000_000n)
}

transfer()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error("Error:", error)
        process.exit(1)
    })
