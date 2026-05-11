import { ApiPromise } from "@polkadot/api"
import { SubmittableExtrinsic } from "@polkadot/api/types"
import { blake2AsHex, xxhashAsHex } from "@polkadot/util-crypto"
import { u8aToHex, hexToU8a } from "@polkadot/util"

// Polkadot BridgeHub para ID. Hardcoded to Polkadot per scope.
const BRIDGE_HUB_POLKADOT_ID = 1002

// Buffer + caps for fallback_max_weight on each BH Transact, matching
// control/preimage/src/helpers.rs:increase_weight.
const MAX_REF_TIME = 60_000_000_000n
const MAX_PROOF_SIZE = 1_048_576n

export interface HaltBridgeOptions {
    gateway?: boolean
    gatewayV2?: boolean
    inboundQueue?: boolean
    inboundQueueV1?: boolean
    inboundQueueV2?: boolean
    outboundQueue?: boolean
    systemFrontend?: boolean
    ethereumClient?: boolean
    assethubMaxFee?: boolean
    assethubMaxFeeV2?: boolean
    all?: boolean
}

export interface HaltBridgePreimage {
    /** Blake2-256 hash of the encoded call. Submit this to the Whitelisted Caller Track. */
    hash: string
    /** SCALE-encoded outer call bytes (0x-prefixed). */
    callData: string
    /** Length in bytes of the encoded call. */
    encodedSize: number
    /** Polkadot.js Apps deep-link that decodes `callData` against AssetHub. */
    decodeUrl: string
    /** Human-readable bullets describing what each halt lever does. */
    summary: string[]
}

/**
 * Build a preimage for halting parts of the Snowbridge V1/V2 stack on Polkadot.
 *
 * Mirrors `control/preimage/src/main.rs::HaltBridge` and produces byte-identical
 * call data (using `pallet_utility::force_batch` so every lever fires
 * independently — see `commands.rs` for the rationale).
 *
 * @param assetHub  Connected ApiPromise for AssetHub-Polkadot.
 * @param bridgeHub Connected ApiPromise for BridgeHub-Polkadot.
 * @param opts      Which halt levers to apply. If all entries are false/undefined,
 *                  `all` is treated as true (matches the CLI's default).
 * @param assetHubWsUrl Optional override for the WS URL embedded in `decodeUrl`.
 *                  Defaults to the public AssetHub-Polkadot RPC.
 */
export async function buildHaltBridgePreimage(
    assetHub: ApiPromise,
    bridgeHub: ApiPromise,
    opts: HaltBridgeOptions,
    assetHubWsUrl: string = "wss://polkadot-asset-hub-rpc.polkadot.io",
): Promise<HaltBridgePreimage> {
    const haltAll = isHaltAllImplied(opts)

    const bhCalls: SubmittableExtrinsic<"promise">[] = []
    const ahCalls: SubmittableExtrinsic<"promise">[] = []
    const summary: string[] = []

    // Gateway halt commands must be enqueued BEFORE any local outbound-queue
    // halt takes effect, otherwise the SetOperatingMode XCM cannot be committed
    // for delivery to Ethereum.
    if (opts.gateway || haltAll) {
        bhCalls.push(bridgeHub.tx.ethereumSystem.setOperatingMode("RejectingOutboundMessages"))
        bhCalls.push(bridgeHub.tx.ethereumSystemV2.setOperatingMode("RejectingOutboundMessages"))
        summary.push("Halt Ethereum Gateway (V1 + V2 paths) — sends SetOperatingMode commands to the Gateway contract.")
    } else if (opts.gatewayV2) {
        bhCalls.push(bridgeHub.tx.ethereumSystemV2.setOperatingMode("RejectingOutboundMessages"))
        summary.push("Halt Ethereum Gateway V2 only — blocks `v2_sendMessage`/`v2_registerToken` once delivered.")
    }

    if (opts.inboundQueue || opts.inboundQueueV1 || haltAll) {
        bhCalls.push(bridgeHub.tx.ethereumInboundQueue.setOperatingMode("Halted"))
        summary.push("Halt V1 inbound-queue on BridgeHub (blocks V1 Ethereum → Polkadot delivery).")
    }
    if (opts.inboundQueue || opts.inboundQueueV2 || haltAll) {
        bhCalls.push(bridgeHub.tx.ethereumInboundQueueV2.setOperatingMode("Halted"))
        summary.push("Halt V2 inbound-queue on BridgeHub (blocks V2 Ethereum → Polkadot delivery).")
    }

    if (opts.outboundQueue || haltAll) {
        bhCalls.push(bridgeHub.tx.ethereumOutboundQueue.setOperatingMode("Halted"))
        ahCalls.push(assetHub.tx.snowbridgeSystemFrontend.setOperatingMode("Halted"))
        summary.push("Halt V1 outbound-queue on BridgeHub + AssetHub system-frontend (router-layer block for both V1 and V2).")
    } else if (opts.systemFrontend) {
        ahCalls.push(assetHub.tx.snowbridgeSystemFrontend.setOperatingMode("Halted"))
        summary.push("Halt AssetHub system-frontend only (V2 router-layer P→E block; V1 continues).")
    }

    if (opts.ethereumClient || haltAll) {
        bhCalls.push(bridgeHub.tx.ethereumBeaconClient.setOperatingMode("Halted"))
        summary.push("Halt Ethereum beacon light client on BridgeHub (blocks new finality ingestion).")
    }

    if (opts.assethubMaxFee || haltAll) {
        ahCalls.push(setAssetHubFeeCall(assetHub, ":BridgeHubEthereumBaseFee:", MAX_U128))
        ahCalls.push(setAssetHubFeeCall(assetHub, ":BridgeHubEthereumBaseFeeV2:", MAX_U128))
        summary.push("Set AssetHub outbound fee = u128::MAX for both V1 and V2 storage items.")
    } else if (opts.assethubMaxFeeV2) {
        ahCalls.push(setAssetHubFeeCall(assetHub, ":BridgeHubEthereumBaseFeeV2:", MAX_U128))
        summary.push("Set AssetHub V2 outbound fee = u128::MAX (V1 fee untouched).")
    }

    if (bhCalls.length === 0 && ahCalls.length === 0) {
        throw new Error("buildHaltBridgePreimage: no levers selected (and `all` was not set).")
    }

    // Wrap AH calls with force_batch if there are multiple, otherwise the single call directly.
    const ahCall =
        ahCalls.length === 0
            ? null
            : ahCalls.length === 1
                ? ahCalls[0]
                : assetHub.tx.utility.forceBatch(ahCalls)

    // Wrap BH calls into a single AH-side `polkadotXcm.send` call. Each BH Transact
    // is preceded by a weight query and followed by ExpectTransactStatus(Success)
    // (matching control/preimage/src/helpers.rs::send_xcm_bridge_hub).
    const bhXcmSend = bhCalls.length === 0
        ? null
        : await wrapBridgeHubCallsInXcmSend(assetHub, bridgeHub, bhCalls)

    // Outer wrap: force_batch so a failure in one (e.g. HRMP transport on the BH
    // XCM-send) does not skip the AH-side halts.
    let outer: SubmittableExtrinsic<"promise">
    if (bhXcmSend && ahCall) {
        outer = assetHub.tx.utility.forceBatch([bhXcmSend, ahCall])
    } else if (bhXcmSend) {
        outer = bhXcmSend
    } else if (ahCall) {
        outer = ahCall
    } else {
        // Already guarded above.
        throw new Error("unreachable")
    }

    const callData = outer.method.toHex()
    const hash = blake2AsHex(callData, 256)
    const encodedSize = (callData.length - 2) / 2 // strip 0x, /2 for byte count

    const decodeUrl =
        `https://polkadot.js.org/apps/?rpc=${encodeURIComponent(assetHubWsUrl)}` +
        `#/extrinsics/decode/${callData}`

    return { hash, callData, encodedSize, decodeUrl, summary }
}

const MAX_U128 = (1n << 128n) - 1n

/**
 * Whether the option set should be treated as "halt everything" — true when the
 * caller passed `all`, or when no individual flag was set (matches CLI behaviour).
 */
function isHaltAllImplied(opts: HaltBridgeOptions): boolean {
    if (opts.all) return true
    return !(
        opts.gateway ||
        opts.gatewayV2 ||
        opts.inboundQueue ||
        opts.inboundQueueV1 ||
        opts.inboundQueueV2 ||
        opts.outboundQueue ||
        opts.systemFrontend ||
        opts.ethereumClient ||
        opts.assethubMaxFee ||
        opts.assethubMaxFeeV2
    )
}

/**
 * Build a `frame_system::set_storage` call writing `u128::MAX` (or any value) to
 * the storage key `twox_128(prefix)` on AssetHub.
 */
function setAssetHubFeeCall(
    assetHub: ApiPromise,
    storageKeyPrefix: string,
    value: bigint,
): SubmittableExtrinsic<"promise"> {
    const key = xxhashAsHex(storageKeyPrefix, 128, true)
    const encodedValue = u8aToHex(assetHub.createType("u128", value.toString()).toU8a())
    return assetHub.tx.system.setStorage([[key, encodedValue]])
}

/**
 * Wrap a list of BH calls into a single AH `pallet_xcm::send` call. Each BH call
 * becomes a `Transact { OriginKind: Superuser, fallbackMaxWeight: 2× queried }`
 * with `ExpectTransactStatus(Success)` after it.
 */
async function wrapBridgeHubCallsInXcmSend(
    assetHub: ApiPromise,
    bridgeHub: ApiPromise,
    bhCalls: SubmittableExtrinsic<"promise">[],
): Promise<SubmittableExtrinsic<"promise">> {
    const instructions: any[] = [
        {
            UnpaidExecution: {
                weightLimit: "Unlimited",
                checkOrigin: null,
            },
        },
    ]

    for (const call of bhCalls) {
        const info: any = await bridgeHub.call.transactionPaymentCallApi.queryCallInfo(
            call.method,
            0,
        )
        const refTime = bigMin(BigInt(info.weight.refTime.toString()) * 2n, MAX_REF_TIME)
        const proofSize = bigMin(BigInt(info.weight.proofSize.toString()) * 2n, MAX_PROOF_SIZE)

        instructions.push({
            Transact: {
                originKind: "Superuser",
                fallbackMaxWeight: { refTime: refTime.toString(), proofSize: proofSize.toString() },
                call: { encoded: u8aToHex(call.method.toU8a()) },
            },
        })
        instructions.push({
            ExpectTransactStatus: { Success: null },
        })
    }

    const dest = {
        V5: {
            parents: 1,
            interior: { X1: [{ Parachain: BRIDGE_HUB_POLKADOT_ID }] },
        },
    }
    const message = { V5: instructions }

    return assetHub.tx.polkadotXcm.send(dest, message)
}

function bigMin(a: bigint, b: bigint): bigint {
    return a < b ? a : b
}
