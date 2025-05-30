import "dotenv/config"
import { Direction } from "@snowbridge/api/dist/forKusama"
import { transferForKusama } from "./transfer_for_kusama"

transferForKusama(
    "# Asset Hub Kusama to Asset Hub Polkadot",
    Direction.ToPolkadot,
    200000000000000n,
    "ETH"
)
    .then(() => process.exit(0))
    .catch((error) => {
        console.error("Error:", error)
        process.exit(1)
    })
