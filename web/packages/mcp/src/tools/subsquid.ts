// Mirrors @snowbridge/api/src/subsquid_v2.ts queries. Inlined so we don't
// import @snowbridge/api into MCP-tool files (its type graph is heavy enough
// to interact badly with the MCP SDK's generic inference).

const TO_POLKADOT_FIELDS = `
    id status blockNumber channelId destinationAddress destinationParaId
    sourceNetwork sourceParaId destinationNetwork l2ChainId messageId nonce
    senderAddress timestamp tokenAddress txHash amount fee
    toBridgeHubInboundQueue { id timestamp txHash channelId nonce messageId }
    toAssetHubMessageQueue { id success timestamp }
    toDestination { id eventId messageId timestamp blockNumber paraId success }
`

const TO_ETHEREUM_FIELDS = `
    id status blockNumber channelId destinationAddress messageId nonce
    senderAddress timestamp tokenAddress amount
    toBridgeHubMessageQueue { id success timestamp }
    toBridgeHubOutboundQueue { id timestamp messageId nonce }
    toDestination { id messageId timestamp blockNumber success }
`

function nonceFilter(id: string): string {
    return id.length > 0 && !id.startsWith("0x") && !isNaN(Number(id))
        ? `{nonce_eq: ${id}}`
        : ""
}

async function queryGraphQL(url: string, query: string): Promise<any> {
    const res = await fetch(url, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ query }),
    })
    if (!res.ok) {
        throw new Error(`subsquid HTTP ${res.status}: ${await res.text()}`)
    }
    const body = (await res.json()) as { data?: any; errors?: any[] }
    if (body.errors?.length) {
        throw new Error(`subsquid GraphQL errors: ${JSON.stringify(body.errors)}`)
    }
    return body.data
}

export async function fetchTransfer(
    url: string,
    direction: "toPolkadot" | "toEthereum",
    id: string,
): Promise<any | undefined> {
    const filter = nonceFilter(id)
    const query =
        direction === "toPolkadot"
            ? `query { transferStatusToPolkadotV2s(limit: 1, where: { OR: [ {messageId_eq: "${id}"} {txHash_eq: "${id}"} ${filter} ] }) { ${TO_POLKADOT_FIELDS} } }`
            : `query { transferStatusToEthereumV2s(limit: 1, where: { OR: [ {messageId_eq: "${id}"} {txHash_eq: "${id}"} ${filter} ] }) { ${TO_ETHEREUM_FIELDS} } }`
    const data = await queryGraphQL(url, query)
    const transfers =
        direction === "toPolkadot"
            ? data?.transferStatusToPolkadotV2s
            : data?.transferStatusToEthereumV2s
    return transfers && transfers.length > 0 ? transfers[0] : undefined
}
