import type { VolumeFeeParams } from "@snowbridge/api"

type FeeAsset = { amount: bigint; symbol: string }

// Optional VolumeFeeParams from env, all three required together:
// VOLUME_FEE_TX_VALUE_USD (whole USD), VOLUME_FEE_ETH_USD_PRICE (whole USD per ETH)
// and SERVICE_FEE_RECIPIENT (Asset Hub AccountId32, SS58 or hex), the account the
// fee is deposited to.
export function volumeFeeFromEnv(): VolumeFeeParams | undefined {
    const txValueUsd = process.env["VOLUME_FEE_TX_VALUE_USD"]
    const ethUsdPrice = process.env["VOLUME_FEE_ETH_USD_PRICE"]
    const serviceFeeRecipient = process.env["SERVICE_FEE_RECIPIENT"]
    if (!txValueUsd || !ethUsdPrice || !serviceFeeRecipient) {
        return undefined
    }
    return {
        txValueUsd: BigInt(txValueUsd),
        ethToUsdNumerator: BigInt(ethUsdPrice),
        ethToUsdDenominator: 1n,
        serviceFeeRecipient,
    }
}

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
