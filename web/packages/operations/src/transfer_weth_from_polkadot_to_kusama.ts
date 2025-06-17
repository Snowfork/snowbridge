import "dotenv/config"
import { Direction } from "@snowbridge/api/dist/forKusama"
import { transferForKusama } from "./transfer_for_kusama"

transferForKusama(
    "# Transfer Weth from Asset Hub Polkadot to Asset Hub Kusama",
    Direction.ToKusama,
    200000000000000n,
    "WETH"
)
    .then(() => process.exit(0))
    .catch((error) => {
        console.error("Error:", error)
        process.exit(1)
    })
