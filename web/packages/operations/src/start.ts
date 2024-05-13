import "dotenv/config"
import { monitor } from "./monitor"

monitor()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error("Error:", error)
        process.exit(1)
    })
