import { DOT_LOCATION } from "./assets_v2"
import {
    bridgeLocation,
    dotLocationOnKusamaAssetHub,
    ksmLocationOnPolkadotAssetHub,
} from "./xcmBuilder"
import { ParachainBase } from "./parachains/parachainBase"

// KSM from Kusama AH's own perspective (parents=1, Here)
const KSM_HERE_LOCATION = { parents: 1, interior: "Here" }

export type PoolReserveCheck =
    | { ok: true }
    | { ok: false; reason: "pool-missing"; pool: string }
    | { ok: false; reason: "insufficient-reserves"; pool: string; reserveOut: bigint; requiredOut: bigint }

// Internal: calls getReserves(asset1, asset2) in the pool's CANONICAL creation order and
// checks the output-side reserve. `asset1`/`asset2` MUST match the on-chain creation order
// (DOT first on Polkadot AH, KSM first on Kusama AH). `outputSide` is the asset coming
// OUT of the actual on-chain swap.
async function checkPoolOutputReserve(
    assetHub: ParachainBase,
    asset1: any,
    asset2: any,
    outputSide: 1 | 2,
    requiredOut: bigint,
    poolLabel: string,
): Promise<PoolReserveCheck> {
    const reserves = await assetHub.getAssetHubPoolReserves(asset1, asset2)
    if (!reserves) {
        return { ok: false, reason: "pool-missing", pool: poolLabel }
    }
    const reserveOut = outputSide === 1 ? reserves.reserve1 : reserves.reserve2
    if (reserveOut <= requiredOut) {
        return {
            ok: false,
            reason: "insufficient-reserves",
            pool: poolLabel,
            reserveOut,
            requiredOut,
        }
    }
    return { ok: true }
}

// DOT/ETH pool on Polkadot AH — Ethereum → Polkadot direction.
// Runtime swap on AH: ETH → DOT (foreign Ether pays DOT-denominated XCM fees).
// Pool creation order: DOT first. Output = DOT = reserve1.
export function checkDotEthPoolLiquidityForEthereumToPolkadot(
    assetHub: ParachainBase,
    ethChainId: number,
    requiredDotOut: bigint,
): Promise<PoolReserveCheck> {
    return checkPoolOutputReserve(
        assetHub,
        DOT_LOCATION,
        bridgeLocation(ethChainId),
        1,
        requiredDotOut,
        "DOT/ETH",
    )
}

// DOT/ETH pool on Polkadot AH — Polkadot → Ethereum direction.
// Runtime swap on AH: DOT → ETH (DOT pays ethereum-side execution fee).
// Pool creation order: DOT first. Output = ETH = reserve2.
export function checkDotEthPoolLiquidityForPolkadotToEthereum(
    assetHub: ParachainBase,
    ethChainId: number,
    requiredEthOut: bigint,
): Promise<PoolReserveCheck> {
    return checkPoolOutputReserve(
        assetHub,
        DOT_LOCATION,
        bridgeLocation(ethChainId),
        2,
        requiredEthOut,
        "DOT/ETH",
    )
}

// <native>/DOT pool on Polkadot AH — parachain (native fee) → Ethereum direction.
// First of two AH swaps when fee is paid in the source parachain's native asset:
// runtime swap on AH is native → DOT, which then funds the DOT → ETH swap.
// Pool creation order: DOT first. Output = DOT = reserve1.
export function checkNativeDotPoolLiquidityForParachainToEthereum(
    assetHub: ParachainBase,
    feeLocation: any,
    requiredDotOut: bigint,
    nativeSymbol?: string,
): Promise<PoolReserveCheck> {
    return checkPoolOutputReserve(
        assetHub,
        DOT_LOCATION,
        feeLocation,
        1,
        requiredDotOut,
        nativeSymbol ? `DOT/${nativeSymbol}` : "DOT/native",
    )
}

// DOT/KSM pool on Polkadot AH — kusama → polkadot direction.
// Swap runs on POLKADOT AH (destination). Runtime swap: KSM → DOT.
// Pool creation order: DOT first. Output = DOT (native) = reserve1.
// Caller must pass the Polkadot AH parachain impl.
export function checkDotKsmPoolLiquidityForKusamaToPolkadot(
    assetHubPolkadot: ParachainBase,
    requiredDotOut: bigint,
): Promise<PoolReserveCheck> {
    return checkPoolOutputReserve(
        assetHubPolkadot,
        DOT_LOCATION,
        ksmLocationOnPolkadotAssetHub,
        1,
        requiredDotOut,
        "DOT/KSM",
    )
}

// KSM/DOT pool on Kusama AH — polkadot → kusama direction.
// Swap runs on KUSAMA AH (destination). Runtime swap: DOT → KSM.
// Pool creation order: KSM first. Output = KSM (native) = reserve1.
// Caller must pass the Kusama AH parachain impl.
export function checkKsmDotPoolLiquidityForPolkadotToKusama(
    assetHubKusama: ParachainBase,
    requiredKsmOut: bigint,
): Promise<PoolReserveCheck> {
    return checkPoolOutputReserve(
        assetHubKusama,
        KSM_HERE_LOCATION,
        dotLocationOnKusamaAssetHub,
        1,
        requiredKsmOut,
        "KSM/DOT",
    )
}
