import "dotenv/config"
import { initializeAlarms } from "./alarm"

initializeAlarms()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error("Error:", error)
        process.exit(1)
    })
