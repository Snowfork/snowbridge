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

/**
 * Query the recent synced blockes on one parachain

curl -H 'Content-Type: application/json' \
-X POST -d \
'{ "query": "query { latestBlockOnChain(id: $id) { height name } }" }' \
$graphqlApiUrl --no-progress-meter | jq "."

{
  "data": {
    "latestBlockOnChain":  {
        "height": 8245566,
        "name": "hydration"
    }
  }
}
**/
export const fetchLatestBlockFromIndexer = async (graphqlApiUrl: string, id: string) => {
    let query = `query { latestBlockOnChain(id: "${id}") {
                    height
                    name
                }}`
    let result = await queryByGraphQL(graphqlApiUrl, query)
    return result && result.latestBlockOnChain && result.latestBlockOnChain[0]
}
