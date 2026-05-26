type FeeBand = {
    lowerUsd: bigint
    upperUsd: bigint
    numerator: bigint
    denominator: bigint
}

const MAX_USD = 999_999_999_999n

const FEE_SCHEDULE: FeeBand[] = [
    { lowerUsd: 0n, upperUsd: 100n, numerator: 16n, denominator: 10_000n },
    { lowerUsd: 100n, upperUsd: 1_000n, numerator: 14n, denominator: 10_000n },
    { lowerUsd: 1_000n, upperUsd: 10_000n, numerator: 12n, denominator: 10_000n },
    { lowerUsd: 10_000n, upperUsd: 100_000n, numerator: 10n, denominator: 10_000n },
    { lowerUsd: 100_000n, upperUsd: 1_000_000n, numerator: 8n, denominator: 10_000n },
    { lowerUsd: 1_000_000n, upperUsd: MAX_USD, numerator: 6n, denominator: 10_000n },
]

export type VolumeFeeParams = {
    txValueUsd: bigint
    ethToUsdNumerator: bigint
    ethToUsdDenominator: bigint
}

export function lookupFeeRatio(txValueUsd: bigint): { numerator: bigint; denominator: bigint } {
    for (const band of FEE_SCHEDULE) {
        if (txValueUsd >= band.lowerUsd && txValueUsd < band.upperUsd) {
            return { numerator: band.numerator, denominator: band.denominator }
        }
    }
    return { numerator: 6n, denominator: 10_000n }
}

export function calculateVolumeTipInWei(params: VolumeFeeParams): bigint {
    if (params.txValueUsd < 0n) throw new Error("txValueUsd must be >= 0")
    if (params.ethToUsdNumerator <= 0n || params.ethToUsdDenominator <= 0n) {
        throw new Error("ethToUsdNumerator and ethToUsdDenominator must be > 0")
    }
    const { numerator: feeNum, denominator: feeDen } = lookupFeeRatio(params.txValueUsd)
    const WEI = 1_000_000_000_000_000_000n
    return (
        (params.txValueUsd * feeNum * WEI * params.ethToUsdDenominator) /
        (feeDen * params.ethToUsdNumerator)
    )
}
