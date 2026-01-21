import "dotenv/config"
import { historyV2, history, subsquid, subsquidV2 } from "@snowbridge/api"

const search = async () => {
    const graphqlApiUrl = process.env["GRAPHQL_API_URL"]
    if (!graphqlApiUrl) throw Error("'GRAPHQL_API_URL' env var not set.")
    await searchV2(graphqlApiUrl)
}

const searchV2 = async (graphqlApiUrl: string) => {
    console.log("To Ethereum transfers:")
    const toEthereums = await historyV2.toEthereumHistory(graphqlApiUrl)
    console.log(JSON.stringify(toEthereums, null, 2))

    console.log("To Polkadot transfers:")
    const toPolkadots = await historyV2.toPolkadotHistory(graphqlApiUrl)
    console.log(JSON.stringify(toPolkadots, null, 2))

    console.log("All transfers order by time:")
    const transfers = [...toEthereums, ...toPolkadots]
    transfers.sort((a, b) => b.info.when.getTime() - a.info.when.getTime())
    console.log(JSON.stringify(transfers, null, 2))

    console.log("To Polkadot transfer by id:")
    const toPolkadot = await historyV2.toPolkadotTransferById(graphqlApiUrl, "7")
    console.log(JSON.stringify(toPolkadot, null, 2))

    console.log("To Ethereum transfer by id:")
    const toEthereum = await historyV2.toEthereumTransferById(graphqlApiUrl, "7")
    console.log(JSON.stringify(toEthereum, null, 2))

    const estimatedDeliveryTime = await subsquidV2.fetchEstimatedDeliveryTime(graphqlApiUrl)
    console.log(estimatedDeliveryTime)

    const latencyToEthereum = await subsquidV2.fetchToEthereumUndeliveredLatency(graphqlApiUrl)
    console.log("To Ethereum V2 undelivered latency:", latencyToEthereum)

    const latencyToPolkadot = await subsquidV2.fetchToPolkadotUndeliveredLatency(graphqlApiUrl)
    console.log("To Polkadot V2 undelivered latency:", latencyToPolkadot)

    const pendingToEthereum = await subsquidV2.fetchToEthereumPendingTransfers(graphqlApiUrl)
    console.log("To Ethereum V2 pending transfers:", pendingToEthereum)

    const pendingToPolkadot = await subsquidV2.fetchToPolkadotPendingTransfers(graphqlApiUrl)
    console.log("To Polkadot V2 pending transfers:", pendingToPolkadot)

    const toEthereumBySenders = await historyV2.toEthereumTransfersBySenders(graphqlApiUrl, 100, [
        "0x7279fcf9694718e1234d102825dccaf332f0ea36edf1ca7c0358c4b68260d24b",
    ])
    console.log("To Ethereum V2 transfers by senders:", toEthereumBySenders)

    const toPolkadotBySenders = await historyV2.toPolkadotTransfersBySenders(graphqlApiUrl, 100, [
        "0xf5bfb6b71d607c0afde874cced435ddd0ae736d1",
    ])
    console.log("To Polkadot V2 transfers by senders:", toPolkadotBySenders)
}

search()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error("Error:", error)
        process.exit(1)
    })
