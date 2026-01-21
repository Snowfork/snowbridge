import { queryByGraphQL } from "./subsquid"

/**
 * Query the recent transfers from Ethereum to Polkadot

```
curl -H 'Content-Type: application/json' \
-X POST -d \
'{ "query": "query { transferStatusToPolkadotV2s(limit: 5, orderBy: blockNumber_DESC) { txHash status channelId destinationAddress messageId nonce senderAddress timestamp tokenAddress amount} }" }' \
$graphqlApiUrl --no-progress-meter | jq "."
```

* @param txHash - the transaction hash on source chain
* @param status - 0:pending, 1: completed 2: failed
* @param messageId - a global index to trace the transfer in different chains
* @param toBridgeHubInboundQueue - transfer received in inbound queue on bridge hub
* @param toAssetHubMessageQueue - transfer received in message queue on asset hub
* @param toDestination - transfer received in message queue on the destination chain, if destination is asset hub then same as toAssetHubMessageQueue
*
"transferStatusToPolkadotV2s": [
      {

        "txHash": "0x53597b6f98334a160f26182398ec3e7368be8ca7aea3eea41d288046f3a1999d",
        "status": 1,
        "channelId": "0xc173fac324158e77fb5840738a1a541f633cbec8884c6a601c567d2b376a0539",
        "destinationAddress": "0x628119c736c0e8ff28bd2f42920a4682bd6feb7b000000000000000000000000",
        "messageId": "0x00d720d39256bab74c0be362005b9a50951a0909e6dabda588a5d319bfbedb65",
        "nonce": 561,
        "senderAddress": "0x628119c736c0e8ff28bd2f42920a4682bd6feb7b",
        "timestamp": "2025-01-20T07:09:47.000000Z",
        "tokenAddress": "0xba41ddf06b7ffd89d1267b5a93bfef2424eb2003",
        "amount": "68554000000000000000000",
        "toBridgeHubInboundQueue:": {"messageId":"0x00d720d39256bab74c0be362005b9a50951a0909e6dabda588a5d319bfbedb65",...},
        "toAssetHubMessageQueue": {"messageId":"0x00d720d39256bab74c0be362005b9a50951a0909e6dabda588a5d319bfbedb65",...},
        "toDestination": {"messageId":"0x00d720d39256bab74c0be362005b9a50951a0909e6dabda588a5d319bfbedb65",...}
      },
      ...
]
 **/
export const fetchToPolkadotTransfers = async (graphqlApiUrl: string, graphqlQuerySize = 100) => {
    let query = `query { transferStatusToPolkadotV2s(limit: ${graphqlQuerySize}, orderBy: timestamp_DESC) {
            id
            status
            blockNumber
            channelId
            destinationAddress
            destinationParaId
            messageId
            nonce
            senderAddress
            timestamp
            tokenAddress
            txHash
            amount
            fee
            sourceNetwork
            destinationNetwork
            sourceParaId
            toBridgeHubInboundQueue {
                id
                timestamp
                txHash
                channelId
                nonce
                messageId
            }
            toAssetHubMessageQueue {
                id
                success
                timestamp
            }
            toDestination {
                id
                eventId
                messageId
                timestamp
                blockNumber
                paraId
                success
            }
        }
    }`
    let result = await queryByGraphQL(graphqlApiUrl, query)
    return result?.transferStatusToPolkadotV2s
}

/**
 * Query the recent transfers from Polkadot to Ethereum

```
curl -H 'Content-Type: application/json' \
-X POST -d \
'{ "query": "query { transferStatusToEthereumV2s(limit: 5, orderBy: blockNumber_DESC) { txHash status channelId destinationAddress messageId nonce senderAddress timestamp tokenAddress amount} }" }' \
$graphqlApiUrl --no-progress-meter | jq "."
```

* @param txHash - the transaction hash on source chain
* @param status - 0:pending, 1: completed 2: failed
* @param messageId - a global index to trace the transfer in different chains
* @param toAssetHubMessageQueue - transfer received in message queue on asset hub
* @param toBridgeHubMessageQueue - transfer received in message queue on bridge hub
* @param toBridgeHubOutboundQueue - transfer received in outbound queue on bridge hub
* @param toDestination - transfer received on the destination chain(Ethereum)
*
"transferStatusToEthereumV2s": [
      {

        "txHash": "0x53597b6f98334a160f26182398ec3e7368be8ca7aea3eea41d288046f3a1999d",
        "status": 1,
        "channelId": "0xc173fac324158e77fb5840738a1a541f633cbec8884c6a601c567d2b376a0539",
        "destinationAddress": "0x628119c736c0e8ff28bd2f42920a4682bd6feb7b000000000000000000000000",
        "messageId": "0x00d720d39256bab74c0be362005b9a50951a0909e6dabda588a5d319bfbedb65",
        "nonce": 561,
        "senderAddress": "0x628119c736c0e8ff28bd2f42920a4682bd6feb7b",
        "timestamp": "2025-01-20T07:09:47.000000Z",
        "tokenAddress": "0xba41ddf06b7ffd89d1267b5a93bfef2424eb2003",
        "amount": "68554000000000000000000",
        "toAssetHubMessageQueue": {"messageId":"0x00d720d39256bab74c0be362005b9a50951a0909e6dabda588a5d319bfbedb65",...},
        "toBridgeHubOutboundQueue:": {"messageId":"0x00d720d39256bab74c0be362005b9a50951a0909e6dabda588a5d319bfbedb65",...},
        "toDestination": {"messageId":"0x00d720d39256bab74c0be362005b9a50951a0909e6dabda588a5d319bfbedb65",...}
      },
      ...
]
 **/
export const fetchToEthereumTransfers = async (graphqlApiUrl: string, graphqlQuerySize = 100) => {
    let query = `query { transferStatusToEthereumV2s(limit: ${graphqlQuerySize}, orderBy: timestamp_DESC) {
            id
            status
            blockNumber
            channelId
            destinationAddress
            messageId
            nonce
            senderAddress
            sourceParaId
            timestamp
            tokenAddress
            txHash
            amount
            fee
            toAssetHubMessageQueue {
                id
                success
                timestamp
            }
            toBridgeHubMessageQueue {
                id
                success
                timestamp
            }
            toBridgeHubOutboundQueue {
                id
                timestamp
            }
            toDestination {
                id
                blockNumber
                timestamp
                txHash
                success
                messageId
                nonce
                channelId
            }
        }
    }`
    let result = await queryByGraphQL(graphqlApiUrl, query)
    return result?.transferStatusToEthereumV2s
}

/**
 * Query transfer from Ethereum to Polkadot by MessageID or TxHash

```
curl -H 'Content-Type: application/json' \
-X POST -d \
'{ "query": "query { transferStatusToPolkadotV2s(where: {messageId_eq: "${id}", OR: {txHash_eq: "${id}"}}) { txHash status channelId destinationAddress messageId nonce senderAddress timestamp tokenAddress amount} }" }' \
$graphqlApiUrl --no-progress-meter | jq "."
```

* @param txHash - the transaction hash on source chain
* @param status - 0:pending, 1: completed 2: failed
* @param messageId - a global index to trace the transfer in different chains
* @param toBridgeHubInboundQueue - transfer received in inbound queue on bridge hub
* @param toAssetHubMessageQueue - transfer received in message queue on asset hub
* @param toDestination - transfer received in message queue on the destination chain, if destination is asset hub then same as toAssetHubMessageQueue
*
"transferStatusToPolkadotV2s": [
      {

        "txHash": "0x53597b6f98334a160f26182398ec3e7368be8ca7aea3eea41d288046f3a1999d",
        "status": 1,
        "channelId": "0xc173fac324158e77fb5840738a1a541f633cbec8884c6a601c567d2b376a0539",
        "destinationAddress": "0x628119c736c0e8ff28bd2f42920a4682bd6feb7b000000000000000000000000",
        "messageId": "0x00d720d39256bab74c0be362005b9a50951a0909e6dabda588a5d319bfbedb65",
        "nonce": 561,
        "senderAddress": "0x628119c736c0e8ff28bd2f42920a4682bd6feb7b",
        "timestamp": "2025-01-20T07:09:47.000000Z",
        "tokenAddress": "0xba41ddf06b7ffd89d1267b5a93bfef2424eb2003",
        "amount": "68554000000000000000000",
        "toBridgeHubInboundQueue:": {"messageId":"0x00d720d39256bab74c0be362005b9a50951a0909e6dabda588a5d319bfbedb65",...},
        "toAssetHubMessageQueue": {"messageId":"0x00d720d39256bab74c0be362005b9a50951a0909e6dabda588a5d319bfbedb65",...},
        "toDestination": {"messageId":"0x00d720d39256bab74c0be362005b9a50951a0909e6dabda588a5d319bfbedb65",...}
      }
]
 **/
export const fetchToPolkadotTransferById = async (graphqlApiUrl: string, id: string) => {
    let nonceFilter =
        id.length > 0 && !id.startsWith("0x") && !isNaN(Number(id)) ? `{nonce_eq: ${id}}` : ""
    let query = `query { transferStatusToPolkadotV2s(limit: 1, where: { OR: [ {messageId_eq: "${id}"} {txHash_eq: "${id}"} ${nonceFilter} ] }) {
            id
            status
            blockNumber
            channelId
            destinationAddress
            destinationParaId
            messageId
            nonce
            senderAddress
            timestamp
            tokenAddress
            txHash
            amount
            fee
            toBridgeHubInboundQueue {
                id
                timestamp
                txHash
                channelId
                nonce
                messageId
            }
            toAssetHubMessageQueue {
                id
                success
                timestamp
            }
            toDestination {
                id
                eventId
                messageId
                timestamp
                blockNumber
                paraId
                success
            }
        }
    }`
    let result = await queryByGraphQL(graphqlApiUrl, query)
    return result?.transferStatusToPolkadotV2s
}

/**
 * Query the transfer from Polkadot to Ethereum by MessageID or TxHash

```
curl -H 'Content-Type: application/json' \
-X POST -d \
'{ "query": "query { transferStatusToEthereumV2s(where: {messageId_eq: "${id}", OR: {txHash_eq: "${id}"}}) { txHash status channelId destinationAddress messageId nonce senderAddress timestamp tokenAddress amount} }" }' \
$graphqlApiUrl --no-progress-meter | jq "."
```

* @param txHash - the transaction hash on source chain
* @param status - 0:pending, 1: completed 2: failed
* @param messageId - a global index to trace the transfer in different chains
* @param toAssetHubMessageQueue - transfer received in message queue on asset hub
* @param toBridgeHubMessageQueue - transfer received in message queue on bridge hub
* @param toBridgeHubOutboundQueue - transfer received in outbound queue on bridge hub
* @param toDestination - transfer received on the destination chain(Ethereum)
*
"transferStatusToEthereumV2s": [
      {

        "txHash": "0x53597b6f98334a160f26182398ec3e7368be8ca7aea3eea41d288046f3a1999d",
        "status": 1,
        "channelId": "0xc173fac324158e77fb5840738a1a541f633cbec8884c6a601c567d2b376a0539",
        "destinationAddress": "0x628119c736c0e8ff28bd2f42920a4682bd6feb7b000000000000000000000000",
        "messageId": "0x00d720d39256bab74c0be362005b9a50951a0909e6dabda588a5d319bfbedb65",
        "nonce": 561,
        "senderAddress": "0x628119c736c0e8ff28bd2f42920a4682bd6feb7b",
        "timestamp": "2025-01-20T07:09:47.000000Z",
        "tokenAddress": "0xba41ddf06b7ffd89d1267b5a93bfef2424eb2003",
        "amount": "68554000000000000000000",
        "toAssetHubMessageQueue": {"messageId":"0x00d720d39256bab74c0be362005b9a50951a0909e6dabda588a5d319bfbedb65",...},
        "toBridgeHubOutboundQueue:": {"messageId":"0x00d720d39256bab74c0be362005b9a50951a0909e6dabda588a5d319bfbedb65",...},
        "toDestination": {"messageId":"0x00d720d39256bab74c0be362005b9a50951a0909e6dabda588a5d319bfbedb65",...}
      }
]
 **/
export const fetchToEthereumTransferById = async (graphqlApiUrl: string, id: string) => {
    let nonceFilter =
        id.length > 0 && !id.startsWith("0x") && !isNaN(Number(id)) ? `{nonce_eq: ${id}}` : ""
    let query = `query { transferStatusToEthereumV2s(limit: 1, where: { OR: [ {messageId_eq: "${id}"} {txHash_eq: "${id}"} ${nonceFilter} ] }) {
            id
            status
            blockNumber
            channelId
            destinationAddress
            messageId
            nonce
            senderAddress
            sourceParaId
            timestamp
            tokenAddress
            txHash
            amount
            fee
            toAssetHubMessageQueue {
                id
                success
                timestamp
            }
            toBridgeHubMessageQueue {
                id
                success
                timestamp
            }
            toBridgeHubOutboundQueue {
                id
                timestamp
            }
            toDestination {
                id
                blockNumber
                timestamp
                txHash
                success
                messageId
                nonce
                channelId
            }
        }
    }`
    let result = await queryByGraphQL(graphqlApiUrl, query)
    return result?.transferStatusToEthereumV2s
}

/**
 * Query the estimated delivery time for transfers to both directions (v2)
 **/
export const fetchEstimatedDeliveryTime = async (graphqlApiUrl: string) => {
    let query = `query { toEthereumV2Elapse { elapse } toPolkadotV2Elapse { elapse } }`
    let result = await queryByGraphQL(graphqlApiUrl, query)
    return result
}

/*
 * Query the maximum latency of pending transfers from V2 P->E.
 * {
    "toEthereumV2UndeliveredTimeout": [
      {
        "elapse": 1034.273011
      }
    ]
}
*/
export const fetchToEthereumUndeliveredLatency = async (graphqlApiUrl: string) => {
    let query = `query { toEthereumV2UndeliveredTimeout {
                   elapse
                }}`
    let result = await queryByGraphQL(graphqlApiUrl, query)
    return result?.toEthereumV2UndeliveredTimeout
}

/* Query the maximum latency of pending transfers from V2 E->P.
 * {
    "toPolkadotV2UndeliveredTimeout": [
      {
        "elapse": 1201.23
      }
    ]
}
*/
export const fetchToPolkadotUndeliveredLatency = async (graphqlApiUrl: string) => {
    let query = `query { toPolkadotV2UndeliveredTimeout {
                   elapse
                }}`
    let result = await queryByGraphQL(graphqlApiUrl, query)
    return result?.toPolkadotV2UndeliveredTimeout
}

// Fetch the pending transfers from Ethereum to Polkadot
export const fetchToPolkadotPendingTransfers = async (
    graphqlApiUrl: string,
    graphqlQuerySize = 100,
) => {
    let query = `query { transferStatusToPolkadotV2s(limit: ${graphqlQuerySize}, orderBy: timestamp_DESC,  where: {status_eq: 0}) {
            id
            status
            blockNumber
            channelId
            destinationAddress
            destinationParaId
            messageId
            nonce
            senderAddress
            timestamp
            tokenAddress
            txHash
            amount
            fee
            sourceNetwork
            destinationNetwork
            sourceParaId
            toBridgeHubInboundQueue {
                id
                timestamp
                txHash
                channelId
                nonce
                messageId
            }
            toAssetHubMessageQueue {
                id
                success
                timestamp
            }
            toDestination {
                id
                eventId
                messageId
                timestamp
                blockNumber
                paraId
                success
            }
        }
    }`
    let result = await queryByGraphQL(graphqlApiUrl, query)
    return result?.transferStatusToPolkadotV2s
}

// Fetch the pending transfers from Polkadot to Ethereum
export const fetchToEthereumPendingTransfers = async (
    graphqlApiUrl: string,
    graphqlQuerySize = 100,
) => {
    let query = `query { transferStatusToEthereumV2s(limit: ${graphqlQuerySize}, orderBy: timestamp_DESC,  where: {status_eq: 0}) {
            id
            status
            blockNumber
            channelId
            destinationAddress
            messageId
            nonce
            senderAddress
            sourceParaId
            timestamp
            tokenAddress
            txHash
            amount
            fee
            toAssetHubMessageQueue {
                id
                success
                timestamp
            }
            toBridgeHubMessageQueue {
                id
                success
                timestamp
            }
            toBridgeHubOutboundQueue {
                id
                timestamp
            }
            toDestination {
                id
                blockNumber
                timestamp
                txHash
                success
                messageId
                nonce
                channelId
            }
        }
    }`
    let result = await queryByGraphQL(graphqlApiUrl, query)
    return result?.transferStatusToEthereumV2s
}

// Fetch the transfers from Ethereum to Polkadot filtered by a list of senders
export const fetchToPolkadotTransfersBySenders = async (
    graphqlApiUrl: string,
    graphqlQuerySize = 100,
    senders: string[],
) => {
    let senderFilter = senders.map((s) => `{senderAddress_eq: "${s}"}`).join(" ")
    let query = `query { transferStatusToPolkadotV2s(limit: ${graphqlQuerySize}, orderBy: timestamp_DESC, where: { OR: [ ${senderFilter} ] }) {
            id
            status
            blockNumber
            channelId
            destinationAddress
            destinationParaId
            messageId
            nonce
            senderAddress
            timestamp
            tokenAddress
            txHash
            amount
            fee
            sourceNetwork
            destinationNetwork
            sourceParaId
            toBridgeHubInboundQueue {
                id
                timestamp
                txHash
                channelId
                nonce
                messageId
            }
            toAssetHubMessageQueue {
                id
                success
                timestamp
            }
            toDestination {
                id
                eventId
                messageId
                timestamp
                blockNumber
                paraId
                success
            }
        }
    }`
    let result = await queryByGraphQL(graphqlApiUrl, query)
    return result?.transferStatusToPolkadotV2s
}

// Fetch the transfers from Polkadot to Ethereum filtered by a list of senders
export const fetchToEthereumTransfersBySenders = async (
    graphqlApiUrl: string,
    graphqlQuerySize = 100,
    senders: string[],
) => {
    let senderFilter = senders.map((s) => `{senderAddress_eq: "${s}"}`).join(" ")
    let query = `query { transferStatusToEthereumV2s(limit: ${graphqlQuerySize}, orderBy: timestamp_DESC,  where: { OR: [ ${senderFilter} ] }) {
            id
            status
            blockNumber
            channelId
            destinationAddress
            messageId
            nonce
            senderAddress
            sourceParaId
            timestamp
            tokenAddress
            txHash
            amount
            fee
            toAssetHubMessageQueue {
                id
                success
                timestamp
            }
            toBridgeHubMessageQueue {
                id
                success
                timestamp
            }
            toBridgeHubOutboundQueue {
                id
                timestamp
            }
            toDestination {
                id
                blockNumber
                timestamp
                txHash
                success
                messageId
                nonce
                channelId
            }
        }
    }`
    let result = await queryByGraphQL(graphqlApiUrl, query)
    return result?.transferStatusToEthereumV2s
}

export const fetchMaxDeliveredNonceToEthereum = async (graphqlApiUrl: string, latest: number) => {
    let query = `query { toEthereumV2LastDelivered(latest: ${latest}) {
                max
            }
        }`
    let result = await queryByGraphQL(graphqlApiUrl, query)
    return result?.toEthereumV2LastDelivered?.max
}

export const fetchMaxDeliveredNonceToPolkadot = async (graphqlApiUrl: string, latest: number) => {
    let query = `query { toPolkadotV2LastDelivered(latest: ${latest}) {
                max
            }
        }`
    let result = await queryByGraphQL(graphqlApiUrl, query)
    return result?.toPolkadotV2LastDelivered?.max
}
