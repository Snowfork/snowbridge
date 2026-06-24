import { FeeEstimateError, FeeEstimateErrorDetails } from "@snowbridge/base-types"

export const estimateFees = async (
    apiEndpoint: string,
    inputToken: string,
    outputToken: string,
    originChainId: number,
    destinationChainId: number,
    amount: bigint,
    // When the deposit carries a cross-chain message (the L2->Polkadot flows
    // execute a swap + Snowbridge `v2_sendMessage` on the destination), pass the
    // multicall handler `recipient` and the `message` so Across includes the
    // message-execution gas in `totalRelayFee`. Without these the API returns
    // the much cheaper plain-transfer gas, the deposit gets underfunded, and no
    // relayer fills it (the deposit expires).
    message?: { recipient: string; message: string },
): Promise<bigint> => {
    const params: Record<string, string> = {
        inputToken,
        outputToken,
        originChainId: originChainId.toString(),
        destinationChainId: destinationChainId.toString(),
        amount: amount.toString(),
    }
    if (message) {
        params.recipient = message.recipient
        params.message = message.message
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
