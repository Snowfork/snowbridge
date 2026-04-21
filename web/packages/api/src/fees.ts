import type { DeliveryFee, FeeAsset, FeeItem } from "./types/fee"

export function addBreakdown<K extends string>(
    breakdown: DeliveryFee<K>["breakdown"],
    key: K,
    asset: FeeAsset,
): void {
    if (asset.amount === 0n) return
    const bucket = (breakdown as Record<string, FeeAsset[]>)[key] ?? []
    ;(breakdown as Record<string, FeeAsset[]>)[key] = bucket
    bucket.push(asset)
}

export function computeTotals(summary: FeeItem[]): FeeAsset[] {
    const totals = new Map<string, bigint>()
    for (const item of summary) {
        totals.set(item.symbol, (totals.get(item.symbol) ?? 0n) + item.amount)
    }
    return [...totals.entries()].map(([symbol, amount]) => ({ symbol, amount }))
}

export function findInBreakdown<K extends string>(
    breakdown: DeliveryFee<K>["breakdown"],
    key: K,
    symbol: string,
): bigint {
    return ((breakdown as Record<string, FeeAsset[]>)[key] ?? [])
        .filter((a) => a.symbol === symbol)
        .reduce((acc, a) => acc + a.amount, 0n)
}

export function findTotal<K extends string>(fee: DeliveryFee<K>, symbol: string): bigint {
    return fee.totals.find((t) => t.symbol === symbol)?.amount ?? 0n
}
