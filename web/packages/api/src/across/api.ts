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

    const url = apiEndpoint + "/suggested-fees?" + new URLSearchParams(params)

    let response: Response
    try {
        response = await fetch(url)
    } catch (error) {
        throw new Error(`Failed to fetch suggested fees from ${url}: ${String(error)}`)
    }

    if (!response.ok) {
        throw new Error(
            `Failed to fetch suggested fees from ${url}: HTTP ${response.status} ${response.statusText}`,
        )
    }

    let data: any
    try {
        data = await response.json()
    } catch (error) {
        throw new Error(`Failed to parse suggested fees response as JSON: ${String(error)}`)
    }

    if (
        !data ||
        typeof data !== "object" ||
        !("totalRelayFee" in data) ||
        !data.totalRelayFee ||
        typeof data.totalRelayFee !== "object" ||
        !("total" in data.totalRelayFee)
    ) {
        throw new Error("Invalid suggested fees response structure: missing totalRelayFee.total")
    }

    const totalValue = (data as any).totalRelayFee.total
    if (
        typeof totalValue !== "string" &&
        typeof totalValue !== "number" &&
        typeof totalValue !== "bigint"
    ) {
        throw new Error("Invalid type for totalRelayFee.total in suggested fees response")
    }

    return BigInt(totalValue)
}
