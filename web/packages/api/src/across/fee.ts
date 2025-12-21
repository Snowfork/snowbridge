export const estimateFees = async (
    feeEndpoint: string,
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
    feeEndpoint += "?" + new URLSearchParams(params)
    let response = await fetch(feeEndpoint)
    let data = await response.json()
    return BigInt(data.fee)
}
