import "dotenv/config"
import { historyV2 } from "@snowbridge/api"

const monitor = async () => {
    const toEthereum = await historyV2.toEthereumHistory()
    console.log(JSON.stringify(toEthereum, null, 2))

    const toPolkadot = await historyV2.toPolkadotHistory()
    console.log(JSON.stringify(toPolkadot, null, 2))

    const transfers = [...toEthereum, ...toPolkadot]
    transfers.sort((a, b) => b.info.when.getTime() - a.info.when.getTime())
    console.log(JSON.stringify(transfers, null, 2))
}

monitor()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error("Error:", error)
        process.exit(1)
    })
