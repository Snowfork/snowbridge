export const estimateFees = async (
    apiEndpoint: string,
    inputToken: string,
    outputToken: string,
    originChainId: number,
    destinationChainId: number,
    amount: bigint,
): Promise<bigint> => {
    const params = {
        inputToken,
        outputToken,
        originChainId: originChainId.toString(),
        destinationChainId: destinationChainId.toString(),
        amount: amount.toString(),
    }
    apiEndpoint += "/suggested-fees?" + new URLSearchParams(params)
    let response = await fetch(apiEndpoint)
    if (!response.ok) {
        throw new Error(`Failed to fetch suggested fees: ${response.status} ${response.statusText}`)
    }
    let data = await response.json()
    return BigInt(data.totalRelayFee.total)
}
