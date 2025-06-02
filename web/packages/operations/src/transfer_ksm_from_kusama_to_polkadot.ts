import "dotenv/config"
import { Direction } from "@snowbridge/api/dist/forKusama"
import { transferForKusama } from "./transfer_for_kusama"

transferForKusama(
    "# Transfer KSM from Asset Hub Kusama to Asset Hub Polkadot",
    Direction.ToPolkadot,
    100000000000n,
    "KSM"
)
    .then(() => process.exit(0))
    .catch((error) => {
        console.error("Error:", error)
        process.exit(1)
    })
