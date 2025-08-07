const graphqlApiUrl = process.env["GRAPHQL_API_URL"] || "https://data.snowbridge.network/graphql"
const graphqlQuerySize = process.env["GRAPHQL_QUERY_SIZE"] || "100"

export const fetchToPolkadotTransfers = async () => {
    let query = `query { transferStatusToPolkadots(limit: ${graphqlQuerySize}, orderBy: blockNumber_DESC) {
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
                success
                timestamp
            }
        }
    }`
    let result = await queryByGraphQL(query)
    return result?.transferStatusToPolkadots
}

export const fetchToEthereumTransfers = async () => {
    let query = `query { transferStatusToEthereums(limit: ${graphqlQuerySize}, orderBy: blockNumber_DESC) {
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

export const fetchBridgeHubOutboundMessageAccepted = async (messageID: string) => {
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

export const fetchEthereumInboundMessageDispatched = async (messageID: string) => {
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

export const fetchBridgeHubInboundMessageReceived = async (messageID: string) => {
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

export const fetchMessageProcessedOnPolkadot = async (messageID: string) => {
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

export const fetchEstimatedDeliveryTime = async (channelId: string) => {
    let query = `query { toEthereumElapse(channelId:"${channelId}") { elapse } toPolkadotElapse(channelId:"${channelId}") { elapse } }`
    let result = await queryByGraphQL(query)
    return result
}

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
