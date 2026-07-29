import "dotenv/config"
import { Direction } from "@snowbridge/api/dist/polkadotKusama"
import { transferPolkadotKusama } from "./transfer_polkadot_kusama"

const transfer = async (
    transferName: string,
    direction: number,
    symbol: string,
    amount: bigint,
) => {
    const directionEnum = direction === 1 ? Direction.ToPolkadot : Direction.ToKusama
    await transferPolkadotKusama(transferName, directionEnum, amount, symbol)
}

if (process.argv.length != 6) {
    console.error("Expected arguments: `transferName, direction, symbol, amount`")
    process.exit(1)
}

transfer(process.argv[2], parseInt(process.argv[3]), process.argv[4], BigInt(process.argv[5]))
    .then(() => process.exit(0))
    .catch((error) => {
        console.error("Error:", error)
        process.exit(1)
    })
