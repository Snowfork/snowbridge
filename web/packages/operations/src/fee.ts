type FeeAsset = { amount: bigint; symbol: string }

type DeliveryFeeLike = {
    breakdown: Record<string, FeeAsset[] | undefined>
    totals: FeeAsset[]
}

export function findFeeTotal(fee: DeliveryFeeLike, symbol: string): bigint {
    return fee.totals.find((item) => item.symbol === symbol)?.amount ?? 0n
}

export function findFeeBreakdownTotal(fee: DeliveryFeeLike, key: string, symbol: string): bigint {
    return (fee.breakdown[key] ?? [])
        .filter((item) => item.symbol === symbol)
        .reduce((total, item) => total + item.amount, 0n)
}
