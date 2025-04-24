import "dotenv/config"
import { transferToEthereum } from "./transfer_to_ethereum"

transferToEthereum(2000, "WETH", 1_000_000_000_000n)
    .then(() => process.exit(0))
    .catch((error) => {
        console.error("Error:", error)
        process.exit(1)
    })
