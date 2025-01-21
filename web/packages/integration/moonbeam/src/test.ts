import { executeXcmMessage } from "./index"

executeXcmMessage()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error("Error:", error)
        process.exit(1)
    })
