const graphqlApiUrl =
    process.env["GRAPHQL_API_URL"] ||
    "https://snowbridge.squids.live/snowbridge-subsquid@v2/api/graphql"
const graphqlQuerySize = process.env["GRAPHQL_QUERY_SIZE"] || "100"

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
export const fetchToPolkadotTransfers = async () => {
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
    let result = await queryByGraphQL(query)
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
export const fetchToEthereumTransfers = async () => {
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
    let result = await queryByGraphQL(query)
    return result?.transferStatusToEthereums
}

const fetchBridgeHubOutboundMessageAccepted = async (messageID: string) => {
    let query = `query { outboundMessageAcceptedOnBridgeHubs(where: {messageId_eq:"${messageID}"}) {
            id
            nonce
            blockNumber
            timestamp
        }   
    }`
    let result = await queryByGraphQL(query)
    return result?.outboundMessageAcceptedOnBridgeHubs[0]
}

const fetchEthereumInboundMessageDispatched = async (messageID: string) => {
    let query = `query {inboundMessageDispatchedOnEthereums(where: {messageId_eq: "${messageID}"}) {
            id
            channelId
            blockNumber
            messageId
            nonce
            success
            timestamp
            txHash
        }
    }`
    let result = await queryByGraphQL(query)
    return result?.inboundMessageDispatchedOnEthereums[0]
}

const fetchBridgeHubInboundMessageReceived = async (messageID: string) => {
    let query = `query { inboundMessageReceivedOnBridgeHubs(where: {messageId_eq:"${messageID}"}) {
            id
            channelId
            blockNumber
            messageId
            nonce
            timestamp
        }   
    }`
    let result = await queryByGraphQL(query)
    return result?.inboundMessageReceivedOnBridgeHubs[0]
}

const fetchMessageProcessedOnPolkadot = async (messageID: string) => {
    let query = `query { messageProcessedOnPolkadots(where: {messageId_eq:"${messageID}"}) {
            id
            blockNumber
            messageId
            paraId
            timestamp
            success
        }   
    }`
    let result = await queryByGraphQL(query)
    return result?.messageProcessedOnPolkadots[0]
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
export const fetchEstimatedDeliveryTime = async (channelId: string) => {
    let query = `query { toEthereumElapse(channelId:"${channelId}") { elapse } toPolkadotElapse(channelId:"${channelId}") { elapse } }`
    let result = await queryByGraphQL(query)
    return result
}

/**
 * Query with a raw graphql
 **/
export const queryByGraphQL = async (query: string) => {
    let response = await fetch(graphqlApiUrl, {
        method: "POST",
        headers: {
            "Content-Type": "application/json",
        },
        body: JSON.stringify({
            query,
        }),
    })
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
export const fetchToPolkadotTransferById = async (id: string) => {
    let query = `query { transferStatusToPolkadots(where: {messageId_eq: "${id}", OR: {txHash_eq: "${id}"}}) {
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
    let result = await queryByGraphQL(query)
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
export const fetchToEthereumTransferById = async (id: string) => {
    let query = `query { transferStatusToEthereums(where: {messageId_eq: "${id}", OR: {txHash_eq: "${id}"}}) {
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
    let result = await queryByGraphQL(query)
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
export const fetchLatestBlocksSynced = async () => {
    let query = `query { latestBlocks {
                    height
                    name
                }}`
    let result = await queryByGraphQL(query)
    return result
}
