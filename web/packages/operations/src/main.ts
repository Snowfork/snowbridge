import "dotenv/config"
import { monitor } from "./monitor"
import { initializeAlarms } from "./alarm"
import * as fisherman from "./fisherman"

if (process.argv.length != 3) {
    console.error("Expected one argument with Enum from `monitor|init|fisherman`")
    process.exit(1)
}

if (process.argv[2] == "monitor") {
    monitor()
        .then(() => {
            console.log("One-shot monitoring succeeded.")
            process.exit(0)
        })
        .catch((error) => {
            console.error("Error:", error)
            process.exit(1)
        })
} else if (process.argv[2] == "init") {
    initializeAlarms()
        .then(() => {
            console.log("Initialize alarm rules succeeded.")
            process.exit(0)
        })
        .catch((error) => {
            console.error("Error:", error)
            process.exit(1)
        })
} else if (process.argv[2] == "fisherman") {
    fisherman
        .run()
        .then(() => {
            console.log("One-shot fisherman succeeded.")
            process.exit(0)
        })
        .catch((error) => {
            console.error("Error:", error)
            process.exit(1)
        })
}
