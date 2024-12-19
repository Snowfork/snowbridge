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
        .then(() => {
            console.log("Oneshot monitoring success.")
            process.exit(0)
        })
        .catch((error) => {
            console.error("Error:", error)
            process.exit(1)
        })
} else if (process.argv[2] == "cron") {
    let interval = parseInt(process.env["SCAN_INTERVAL"] || "") || 30
    cron.schedule(`*/${interval} * * * *`, monitor)
    console.log("cron task installed for monitoring with interval:" + interval + " (in minutes)")
} else if (process.argv[2] == "init") {
    initializeAlarms()
        .then(() => {
            console.log("Initialize alarm rules for monitoring success.")
            process.exit(0)
        })
        .catch((error) => {
            console.error("Error:", error)
            process.exit(1)
        })
    
}
