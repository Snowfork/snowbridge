import "dotenv/config"
import cron from "node-cron"
import { transferToPolkadot } from "./transfer_to_polkadot"

const transfer = async () => {
    await transferToPolkadot(1000, "Ether", 10_000_000_000_000n)
}

if (process.argv.length != 3) {
    console.error("Expected one argument with Enum from `start|cron`")
    process.exit(1)
}

if (process.argv[2] == "start") {
    transfer()
        .then(() => process.exit(0))
        .catch((error) => {
            console.error("Error:", error)
            process.exit(1)
        })
} else if (process.argv[2] == "cron") {
    console.log("running cronjob")
    cron.schedule(process.env["TRANSFER_ETHER_CRON_EXPRESSION"] || "0 0 * * *", transfer)
}
