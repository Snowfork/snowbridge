import "dotenv/config"
import { historyV2 } from "@snowbridge/api"

const monitor = async () => {
    console.log("To Ethereum transfers:")
    const toEthereums = await historyV2.toEthereumHistory()
    console.log(JSON.stringify(toEthereums, null, 2))

    console.log("To Polkadot transfers:")
    const toPolkadots = await historyV2.toPolkadotHistory()
    console.log(JSON.stringify(toPolkadots, null, 2))

    console.log("All transfers order by time:")
    const transfers = [...toEthereums, ...toPolkadots]
    transfers.sort((a, b) => b.info.when.getTime() - a.info.when.getTime())
    console.log(JSON.stringify(transfers, null, 2))

    console.log("To Polkadot transfer by id:")
    const toPolkadot = await historyV2.toPolkadotTransferById(
        "0xb56662848712da9769a2122ca0d24d199ef7af7c8aedee43778dadbe1c42ebc6"
    )
    console.log(JSON.stringify(toPolkadot, null, 2))

    console.log("To Ethereum transfer by id:")
    const toEthereum = await historyV2.toEthereumTransferById(
        "0x04b7a6c7552d2890094dfe43e037cb5f5495fec2419f33b0072439a9ee7629a0"
    )
    console.log(JSON.stringify(toEthereum, null, 2))
}

monitor()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error("Error:", error)
        process.exit(1)
    })
