import "dotenv/config"
import cron from "node-cron"
import { monitor } from "./monitor"
import { initializeAlarms } from "./alarm"

if (process.argv.length != 3) {
    console.error("Expected one argument with Enum from `start|cron|init`")
    process.exit(1)
}

if (process.argv[2] == "start") {
    monitor()
        .then(() => process.exit(0))
        .catch((error) => {
            console.error("Error:", error)
            process.exit(1)
        })
} else if (process.argv[2] == "cron") {
    let interval = parseInt(process.env["SCAN_INTERVAL"] || "") || 30
    cron.schedule(`*/${interval} * * * *`, monitor)
} else if (process.argv[2] == "init") {
    initializeAlarms()
        .then(() => process.exit(0))
        .catch((error) => {
            console.error("Error:", error)
            process.exit(1)
        })
}
