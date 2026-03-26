import { FeeEstimateError, FeeEstimateErrorDetails } from "@snowbridge/base-types"

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

    const response = await fetch(url)
    if (!response.ok) {
        const error = await response.json()
        throw new FeeEstimateError(error as FeeEstimateErrorDetails)
    }
    const data = await response.json()

    if (
        !data ||
        typeof data !== "object" ||
        !("totalRelayFee" in data) ||
        !data.totalRelayFee ||
        typeof data.totalRelayFee !== "object" ||
        !("total" in data.totalRelayFee)
    ) {
        throw new Error(
            "Invalid suggested fees response structure: missing totalRelayFee.total: " +
                JSON.stringify(data),
        )
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
