/**
 * Query the recent transfers from Ethereum to Polkadot

```
curl -H 'Content-Type: application/json' \
-X POST -d \
'{ "query": "query { transferStatusToPolkadots(limit: 5, orderBy: blockNumber_DESC) { txHash status channelId destinationAddress messageId nonce senderAddress timestamp tokenAddress amount} }" }' \
$graphqlApiUrl --no-progress-meter | jq "."
```

* @param txHash - the transaction hash on source chain
* @param status - 0:pending, 1: completed 2: failed
* @param messageId - a global index to trace the transfer in different chains
* @param toBridgeHubInboundQueue - transfer received in inbound queue on bridge hub
* @param toAssetHubMessageQueue - transfer received in message queue on asset hub
* @param toDestination - transfer received in message queue on the destination chain, if destination is asset hub then same as toAssetHubMessageQueue
*
"transferStatusToPolkadots": [
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
    let query = `query { transferStatusToPolkadots(limit: ${graphqlQuerySize}, orderBy: timestamp_DESC) {
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
    return result?.transferStatusToPolkadots
}

/**
 * Query the recent transfers from Polkadot to Ethereum

```
curl -H 'Content-Type: application/json' \
-X POST -d \
'{ "query": "query { transferStatusToEthereums(limit: 5, orderBy: blockNumber_DESC) { txHash status channelId destinationAddress messageId nonce senderAddress timestamp tokenAddress amount} }" }' \
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
"transferStatusToEthereums": [
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
    let query = `query { transferStatusToEthereums(limit: ${graphqlQuerySize}, orderBy: timestamp_DESC) {
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
    return result?.transferStatusToEthereums
}

/**
 * Query the estimated delivery time for transfers to both directions

curl -H 'Content-Type: application/json' \
-X POST -d \
'{ "query": "query { toEthereumElapse { elapse } toPolkadotElapse { elapse } }" }' \
$graphqlApiUrl --no-progress-meter | jq "."

* @param elapse - the estimated delivery time of the transfer so far in average (in seconds)

{
  "data": {
    "toEthereumElapse": {
      "elapse": 7521.195804
    },
    "toPolkadotElapse": {
      "elapse": 1197.827338
    }
  }
}
**/
export const fetchEstimatedDeliveryTime = async (graphqlApiUrl: string, channelId: string) => {
    let query = `query { toEthereumElapse(channelId:"${channelId}") { elapse } toPolkadotElapse(channelId:"${channelId}") { elapse } }`
    let result = await queryByGraphQL(graphqlApiUrl, query)
    return result
}

/**
 * Query the estimated delivery time for transfers to both directions (v2)
 **/
export const fetchV2EstimatedDeliveryTime = async (graphqlApiUrl: string) => {
    let query = `query { toEthereumV2Elapse { elapse } toPolkadotV2Elapse { elapse } }`
    let result = await queryByGraphQL(graphqlApiUrl, query)
    return result
}

/**
 * Query with a raw graphql
 **/
export const queryByGraphQL = async (graphqlApiUrl: string, query: string) => {
    let response = await fetch(graphqlApiUrl, {
        method: "POST",
        headers: {
            "Content-Type": "application/json",
        },
        body: JSON.stringify({
            query,
        }),
    })
    // proper error checking
    if (!response.ok) {
        console.error(`${response.status} ${response.statusText}\nBody:`, await response.text())
        throw Error(`Error querying graphql: ${response.status}: ${response.statusText}`)
    }
    let data = await response.json()
    return data?.data
}

/**
 * Query transfer from Ethereum to Polkadot by MessageID or TxHash

```
curl -H 'Content-Type: application/json' \
-X POST -d \
'{ "query": "query { transferStatusToPolkadots(where: {messageId_eq: "${id}", OR: {txHash_eq: "${id}"}}) { txHash status channelId destinationAddress messageId nonce senderAddress timestamp tokenAddress amount} }" }' \
$graphqlApiUrl --no-progress-meter | jq "."
```

* @param txHash - the transaction hash on source chain
* @param status - 0:pending, 1: completed 2: failed
* @param messageId - a global index to trace the transfer in different chains
* @param toBridgeHubInboundQueue - transfer received in inbound queue on bridge hub
* @param toAssetHubMessageQueue - transfer received in message queue on asset hub
* @param toDestination - transfer received in message queue on the destination chain, if destination is asset hub then same as toAssetHubMessageQueue
*
"transferStatusToPolkadots": [
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
    let query = `query { transferStatusToPolkadots(limit: 1, where: {messageId_eq: "${id}", OR: {txHash_eq: "${id}"}}) {
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
    return result?.transferStatusToPolkadots
}

/**
 * Query the transfer from Polkadot to Ethereum by MessageID or TxHash

```
curl -H 'Content-Type: application/json' \
-X POST -d \
'{ "query": "query { transferStatusToEthereums(where: {messageId_eq: "${id}", OR: {txHash_eq: "${id}"}}) { txHash status channelId destinationAddress messageId nonce senderAddress timestamp tokenAddress amount} }" }' \
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
"transferStatusToEthereums": [
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
    let query = `query { transferStatusToEthereums(limit: 1, where: {messageId_eq: "${id}", OR: {txHash_eq: "${id}"}}) {
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
    return result?.transferStatusToEthereums
}

/**
 * Query the recent synced blockes on multiple chains

curl -H 'Content-Type: application/json' \
-X POST -d \
'{ "query": "query { latestBlocks { height name } }" }' \
$graphqlApiUrl --no-progress-meter | jq "."

{
  "data": {
    "latestBlocks": [
      {
        "height": 8245566,
        "name": "assethub"
      },
      {
        "height": 4561260,
        "name": "bridgehub"
      },
      {
        "height": 21878012,
        "name": "ethereum"
      }
    ]
  }
}
**/
export const fetchLatestBlocksSynced = async (graphqlApiUrl: string, includePKBridge: boolean) => {
    let query = `query { latestBlocks(withPKBridge: ${includePKBridge}) {
                    height
                    name
                }}`
    let result = await queryByGraphQL(graphqlApiUrl, query)
    return result && result.latestBlocks
}

/**
 * Query messages processed on parachains.

```
curl -H 'Content-Type: application/json' \
-X POST -d \
'query { messageProcessedOnPolkadots(limit: 1, where: {messageId_eq: "${id}"}) { id blockNumber timestamp messageId paraId success eventId network } }' \
$graphqlApiUrl --no-progress-meter | jq "."
```

* @param id - internal identifier for the message
* @param status - true or false
* @param messageId - a global index to trace the transfer in different chains
* @param timestamp - When the message was processed.
* @param paraId - The parachain the message was processed on.
* @param eventId - The id of the message processed/failed event.
* @param network - The chain it was received on.
*
"messageProcessedOnPolkadots": [
    {
    "id": "0007409855-e807a-000010",
    "blockNumber": 7409855,
    "timestamp": "2024-11-22T15:53:00.000000Z",
    "messageId": "0x67f2e507665b5a22b302f8ed998ff6f40afd967c974457d6610e795776611c85",
    "paraId": 2000,
    "success": true,
    "eventId": "7409855-10",
    "network": null
    }
]
 **/
export const fetchInterParachainMessageById = async (graphqlApiUrl: string, id: string) => {
    let query = `query { messageProcessedOnPolkadots(limit: 1, where: {messageId_eq: "${id}"}) {
                        id
                        blockNumber
                        timestamp
                        messageId
                        paraId
                        success
                        eventId
                        network
                    }
                }`
    let result = await queryByGraphQL(graphqlApiUrl, query)
    return result?.messageProcessedOnPolkadots
}

/*
 * Query the maximum latency of pending transfers from P->E.
 * {
    "toEthereumUndeliveredTimeout": [
      {
        "elapse": 1034.273011
      }
    ]
}
*/
export const fetchToEthereumUndelivedLatency = async (graphqlApiUrl: string) => {
    let query = `query { toEthereumUndeliveredTimeout {
                   elapse
                }}`
    let result = await queryByGraphQL(graphqlApiUrl, query)
    return result?.toEthereumUndeliveredTimeout
}

/* Query the maximum latency of pending transfers from E->P.
 * {
    "toPolkadotUndeliveredTimeout": [
      {
        "elapse": 1201.23
      }
    ]
}
*/
export const fetchToPolkadotUndelivedLatency = async (graphqlApiUrl: string) => {
    let query = `query { toPolkadotUndeliveredTimeout {
                   elapse
                }}`
    let result = await queryByGraphQL(graphqlApiUrl, query)
    return result?.toPolkadotUndeliveredTimeout
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
export const fetchToEthereumV2UndelivedLatency = async (graphqlApiUrl: string) => {
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
export const fetchToPolkadotV2UndelivedLatency = async (graphqlApiUrl: string) => {
    let query = `query { toPolkadotV2UndeliveredTimeout {
                   elapse
                }}`
    let result = await queryByGraphQL(graphqlApiUrl, query)
    return result?.toPolkadotV2UndeliveredTimeout
}

/**
 * Query the recent synced blockes on one parachain

curl -H 'Content-Type: application/json' \
-X POST -d \
'{ "query": "query { latestBlocksOfParachain(paraid: $paraid) { height name paraid } }" }' \
$graphqlApiUrl --no-progress-meter | jq "."

{
  "data": {
    "latestBlocksOfParachain":  {
        "height": 8245566,
        "name": "hydration"
    }
  }
}
**/
export const fetchSyncStatusOfParachain = async (graphqlApiUrl: string, paraid: number) => {
    let query = `query { latestBlocksOfParachain(paraid: ${paraid}) {
                    height
                    name
                    paraid
                }}`
    let result = await queryByGraphQL(graphqlApiUrl, query)
    return result && result.latestBlocksOfParachain && result.latestBlocksOfParachain[0]
}
