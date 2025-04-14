import "dotenv/config"
import { registerERC20 } from "./register_erc20"

registerERC20("WETH")
    .then(() => process.exit(0))
    .catch((error) => {
        console.error("Error:", error)
        process.exit(1)
    })
