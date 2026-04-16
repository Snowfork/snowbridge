type FeeBand = {
    lowerUsd: bigint // inclusive
    upperUsd: bigint // exclusive
    numerator: bigint
    denominator: bigint
}

// Effectively infinity for the last band
const MAX_USD = 999_999_999_999n

// New fee schedule: 0.16/0.14/0.12/0.10/0.08/0.06%
// Bands are strictly: lowerUsd <= txValueUsd < upperUsd
const FEE_SCHEDULE: FeeBand[] = [
    { lowerUsd: 0n, upperUsd: 100n, numerator: 16n, denominator: 10_000n },
    { lowerUsd: 100n, upperUsd: 1_000n, numerator: 14n, denominator: 10_000n },
    { lowerUsd: 1_000n, upperUsd: 10_000n, numerator: 12n, denominator: 10_000n },
    { lowerUsd: 10_000n, upperUsd: 100_000n, numerator: 10n, denominator: 10_000n },
    { lowerUsd: 100_000n, upperUsd: 1_000_000n, numerator: 8n, denominator: 10_000n },
    { lowerUsd: 1_000_000n, upperUsd: MAX_USD, numerator: 6n, denominator: 10_000n },
]

export type VolumeFeeParams = {
    txValueUsd: bigint // USD value of the transaction (whole USD, bigint)
    ethToUsdNumerator: bigint // e.g. 2500n if 1 ETH = $2500
    ethToUsdDenominator: bigint // e.g. 1n (simple ratio). Use denominator > 1 for fractional prices.
}

/**
 * Lookup the fee ratio for a given USD transaction value.
 * Band boundaries: lowerUsd <= txValueUsd < upperUsd
 */
export function lookupFeeRatio(txValueUsd: bigint): { numerator: bigint; denominator: bigint } {
    for (const band of FEE_SCHEDULE) {
        if (txValueUsd >= band.lowerUsd && txValueUsd < band.upperUsd) {
            return { numerator: band.numerator, denominator: band.denominator }
        }
    }
    // Fallback to the largest band (>= $1M)
    return { numerator: 6n, denominator: 10_000n }
}

/**
 * Calculate the volume-based fee tip denominated in Ether (wei).
 *
 * Formula:
 *   tipInWei = txValueUsd * feeNumerator * 1e18 * ethToUsdDenominator
 *              / (feeDenominator * ethToUsdNumerator)
 *
 * All arithmetic is bigint. Result is floored (best-case tip for the user).
 *
 * @param txValueUsd - USD value of the transaction (whole USD as bigint, e.g. 5000n for $5000)
 * @param ethToUsdNumerator - numerator of ETH price in USD (e.g. 2500n for $2500/ETH)
 * @param ethToUsdDenominator - denominator of ETH price in USD (e.g. 1n for whole dollars, 100n for cents)
 */
export function calculateVolumeTipInWei(
    txValueUsd: bigint,
    ethToUsdNumerator: bigint,
    ethToUsdDenominator: bigint,
): bigint {
    if (txValueUsd < 0n) {
        throw new Error("txValueUsd must be >= 0")
    }
    if (ethToUsdNumerator <= 0n) {
        throw new Error("ethToUsdNumerator must be > 0")
    }
    if (ethToUsdDenominator <= 0n) {
        throw new Error("ethToUsdDenominator must be > 0")
    }

    const { numerator: feeNum, denominator: feeDen } = lookupFeeRatio(txValueUsd)
    const WEI = 1_000_000_000_000_000_000n

    // Multiply all numerators first to minimize integer truncation
    return (txValueUsd * feeNum * WEI * ethToUsdDenominator) / (feeDen * ethToUsdNumerator)
}
