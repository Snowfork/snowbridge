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
    gatewayV1?: boolean
    gatewayV2?: boolean
    inboundQueue?: boolean
    inboundQueueV1?: boolean
    inboundQueueV2?: boolean
    outboundQueue?: boolean
    outboundQueueV1?: boolean
    systemFrontend?: boolean
    ethereumClient?: boolean
    assethubMaxFee?: boolean
    assethubMaxFeeV1?: boolean
    assethubMaxFeeV2?: boolean
    all?: boolean
}

export interface StorageWrite {
    /** Human-readable parameter name (matches the runtime `parameter_types!` entry). */
    name: string
    /** twox_128 hash of `:NAME:`,the actual storage key being written. */
    key: string
    /** SCALE-encoded value being written, 0x-prefixed. */
    value: string
    /** Link to the runtime source declaring this parameter, when known. */
    sourceUrl?: string
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
    /**
     * Raw `frame_system::set_storage` writes included in the preimage. Each entry
     * lets a reviewer verify that the bytes in the call data correspond to a
     * known runtime parameter (the key is just `twox_128(":NAME:")`).
     */
    storageWrites: StorageWrite[]
}

const FEE_SOURCE_URL =
    "https://github.com/polkadot-fellows/runtimes/blob/main/system-parachains/asset-hubs/asset-hub-polkadot/src/xcm_config.rs"

/**
 * Build a preimage for halting parts of the Snowbridge V1/V2 stack on Polkadot.
 *
 * Mirrors `control/preimage/src/main.rs::HaltBridge` and produces byte-identical
 * call data (using `pallet_utility::force_batch` so every lever fires
 * independently,see `commands.rs` for the rationale).
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
    const storageWrites: StorageWrite[] = []

    const pushFeeWrite = (name: string) => {
        const { call, write } = setAssetHubFeeCallWithWrite(assetHub, name, MAX_U128)
        ahCalls.push(call)
        storageWrites.push(write)
    }

    // Gateway halt commands ride the outbound queues: V1 SetOperatingMode goes
    // through `EthereumOutboundQueue` on the PRIMARY_GOVERNANCE_CHANNEL (which
    // bypasses the V1 outbound-queue halt,see
    // `pallets/outbound-queue/src/send_message_impl.rs:79`), and V2 goes through
    // `EthereumOutboundQueueV2` (which has no halt lever at all). Ordering them
    // first in the BH XCM is defensive,strictly unnecessary today.
    if (opts.gateway || haltAll) {
        bhCalls.push(bridgeHub.tx.ethereumSystem.setOperatingMode("RejectingOutboundMessages"))
        bhCalls.push(bridgeHub.tx.ethereumSystemV2.setOperatingMode("RejectingOutboundMessages"))
        summary.push("Halt Ethereum Gateway (V1 + V2 paths),sends SetOperatingMode commands to the Gateway contract.")
    } else {
        if (opts.gatewayV1) {
            bhCalls.push(bridgeHub.tx.ethereumSystem.setOperatingMode("RejectingOutboundMessages"))
            summary.push("Halt Ethereum Gateway V1 only,blocks V1 `sendToken`/`sendMessage` once delivered.")
        }
        if (opts.gatewayV2) {
            bhCalls.push(bridgeHub.tx.ethereumSystemV2.setOperatingMode("RejectingOutboundMessages"))
            summary.push("Halt Ethereum Gateway V2 only,blocks `v2_sendMessage`/`v2_registerToken` once delivered.")
        }
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
        summary.push("Halt V1 outbound-queue on BridgeHub (rejects V1 user/sibling P→E; V1 governance channel still bypasses the halt).")
        summary.push("Halt AssetHub system-frontend,blocks both V1 and V2 P→E at the router layer.")
    } else {
        if (opts.outboundQueueV1) {
            bhCalls.push(bridgeHub.tx.ethereumOutboundQueue.setOperatingMode("Halted"))
            summary.push("Halt V1 outbound-queue on BridgeHub only (rejects V1 user/sibling P→E; V1 governance channel still bypasses the halt; AH frontend untouched so V1+V2 P→E still enter the router).")
        }
        if (opts.systemFrontend) {
            ahCalls.push(assetHub.tx.snowbridgeSystemFrontend.setOperatingMode("Halted"))
            summary.push("Halt AssetHub system-frontend,blocks both V1 and V2 P→E at the router layer (V1 BH outbound-queue keeps draining in-flight messages).")
        }
    }

    if (opts.ethereumClient || haltAll) {
        bhCalls.push(bridgeHub.tx.ethereumBeaconClient.setOperatingMode("Halted"))
        summary.push("Halt Ethereum beacon light client on BridgeHub (blocks new finality ingestion).")
    }

    if (opts.assethubMaxFee || haltAll) {
        pushFeeWrite("BridgeHubEthereumBaseFee")
        pushFeeWrite("BridgeHubEthereumBaseFeeV2")
        summary.push("Set AssetHub outbound fee = u128::MAX for both V1 and V2 storage items.")
    } else {
        if (opts.assethubMaxFeeV1) {
            pushFeeWrite("BridgeHubEthereumBaseFee")
            summary.push("Set AssetHub V1 outbound fee = u128::MAX (V2 fee untouched).")
        }
        if (opts.assethubMaxFeeV2) {
            pushFeeWrite("BridgeHubEthereumBaseFeeV2")
            summary.push("Set AssetHub V2 outbound fee = u128::MAX (V1 fee untouched).")
        }
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

    return { hash, callData, encodedSize, decodeUrl, summary, storageWrites }
}

const MAX_U128 = (1n << 128n) - 1n

// Production base-fee values captured 2026-05-18 from
// wss://polkadot-asset-hub-rpc.polkadot.io. These are the storage values the
// resume preimage writes back to `:BridgeHubEthereumBaseFee:` /
// `:BridgeHubEthereumBaseFeeV2:` to restore the current governance-tuned fee
// policy after a halt sets them to u128::MAX. Raw LE bytes:
//   :BridgeHubEthereumBaseFee:   = 0x86b7de79030000000000000000000000
//   :BridgeHubEthereumBaseFeeV2: = 0x00ca9a3b000000000000000000000000
//
// IMPORTANT: do NOT pull these from the runtime source defaults in
// polkadot-fellows/runtimes
// (system-parachains/asset-hubs/asset-hub-polkadot/src/xcm_config.rs ~line 802:
// `DefaultBridgeHubEthereumBaseFee = Balance::MAX`). The source defaults are
// intentionally `u128::MAX` to ship the bridge in a halted state; the live
// values were set by past governance via `system.setStorage` and are what we
// want to restore on resume.
//
// Exported so callers (e.g. the snowbridge-app governance UI) can prefill
// fee inputs and let the operator confirm or override them at submission
// time. If the fee policy is ever updated on-chain, refresh these constants
// from the same storage keys.
export const PROD_BASE_FEE_V1 = 14_929_540_998n
export const PROD_BASE_FEE_V2 = 1_000_000_000n

export interface ResumeBridgeOptions {
    gateway?: boolean
    gatewayV1?: boolean
    gatewayV2?: boolean
    inboundQueue?: boolean
    inboundQueueV1?: boolean
    inboundQueueV2?: boolean
    outboundQueue?: boolean
    outboundQueueV1?: boolean
    systemFrontend?: boolean
    ethereumClient?: boolean
    assethubBaseFee?: boolean
    assethubBaseFeeV1?: boolean
    assethubBaseFeeV2?: boolean
    /**
     * Override the V1 base-fee value written when the assethubBaseFee or
     * assethubBaseFeeV1 (or `all`) levers are active. Defaults to
     * {@link PROD_BASE_FEE_V1}.
     */
    baseFeeV1?: bigint
    /**
     * Override the V2 base-fee value written when the assethubBaseFee or
     * assethubBaseFeeV2 (or `all`) levers are active. Defaults to
     * {@link PROD_BASE_FEE_V2}.
     */
    baseFeeV2?: bigint
    all?: boolean
}

/**
 * Build a preimage for resuming (un-halting) the Snowbridge V1/V2 stack on
 * Polkadot. Inverse of `buildHaltBridgePreimage`: queues `setOperatingMode`
 * calls back to `Normal` / `RejectingOutboundMessages` -> `Normal`, and
 * restores the AssetHub base fees from u128::MAX to their pre-halt prod
 * values (see {@link PROD_BASE_FEE_V1} / {@link PROD_BASE_FEE_V2}).
 *
 * The structure (force_batch, polkadotXcm.send wrapping BH Transacts, weight
 * queries with 2x cap, ExpectTransactStatus(Success) after each Transact) is
 * identical to the halt preimage.
 *
 * @param assetHub  Connected ApiPromise for AssetHub-Polkadot.
 * @param bridgeHub Connected ApiPromise for BridgeHub-Polkadot.
 * @param opts      Which resume levers to apply. If all entries are
 *                  false/undefined, `all` is treated as true.
 * @param assetHubWsUrl Optional override for the WS URL embedded in `decodeUrl`.
 */
export async function buildResumeBridgePreimage(
    assetHub: ApiPromise,
    bridgeHub: ApiPromise,
    opts: ResumeBridgeOptions,
    assetHubWsUrl: string = "wss://polkadot-asset-hub-rpc.polkadot.io",
): Promise<HaltBridgePreimage> {
    const resumeAll = isResumeAllImplied(opts)

    const bhCalls: SubmittableExtrinsic<"promise">[] = []
    const ahCalls: SubmittableExtrinsic<"promise">[] = []
    const summary: string[] = []
    const storageWrites: StorageWrite[] = []

    const pushFeeWrite = (name: string, value: bigint) => {
        const { call, write } = setAssetHubFeeCallWithWrite(assetHub, name, value)
        ahCalls.push(call)
        storageWrites.push(write)
    }

    if (opts.gateway || resumeAll) {
        bhCalls.push(bridgeHub.tx.ethereumSystem.setOperatingMode("Normal"))
        bhCalls.push(bridgeHub.tx.ethereumSystemV2.setOperatingMode("Normal"))
        summary.push("Resume Ethereum Gateway (V1 + V2 paths), sends SetOperatingMode(Normal) commands to the Gateway contract.")
    } else {
        if (opts.gatewayV1) {
            bhCalls.push(bridgeHub.tx.ethereumSystem.setOperatingMode("Normal"))
            summary.push("Resume Ethereum Gateway V1 only.")
        }
        if (opts.gatewayV2) {
            bhCalls.push(bridgeHub.tx.ethereumSystemV2.setOperatingMode("Normal"))
            summary.push("Resume Ethereum Gateway V2 only.")
        }
    }

    if (opts.inboundQueue || opts.inboundQueueV1 || resumeAll) {
        bhCalls.push(bridgeHub.tx.ethereumInboundQueue.setOperatingMode("Normal"))
        summary.push("Resume V1 inbound-queue on BridgeHub.")
    }
    if (opts.inboundQueue || opts.inboundQueueV2 || resumeAll) {
        bhCalls.push(bridgeHub.tx.ethereumInboundQueueV2.setOperatingMode("Normal"))
        summary.push("Resume V2 inbound-queue on BridgeHub.")
    }

    if (opts.outboundQueue || resumeAll) {
        bhCalls.push(bridgeHub.tx.ethereumOutboundQueue.setOperatingMode("Normal"))
        ahCalls.push(assetHub.tx.snowbridgeSystemFrontend.setOperatingMode("Normal"))
        summary.push("Resume V1 outbound-queue on BridgeHub.")
        summary.push("Resume AssetHub system-frontend.")
    } else {
        if (opts.outboundQueueV1) {
            bhCalls.push(bridgeHub.tx.ethereumOutboundQueue.setOperatingMode("Normal"))
            summary.push("Resume V1 outbound-queue on BridgeHub only.")
        }
        if (opts.systemFrontend) {
            ahCalls.push(assetHub.tx.snowbridgeSystemFrontend.setOperatingMode("Normal"))
            summary.push("Resume AssetHub system-frontend only.")
        }
    }

    if (opts.ethereumClient || resumeAll) {
        bhCalls.push(bridgeHub.tx.ethereumBeaconClient.setOperatingMode("Normal"))
        summary.push("Resume Ethereum beacon light client on BridgeHub.")
    }

    const feeV1 = opts.baseFeeV1 ?? PROD_BASE_FEE_V1
    const feeV2 = opts.baseFeeV2 ?? PROD_BASE_FEE_V2
    const v1Note =
        opts.baseFeeV1 === undefined ? "default" : "caller-supplied override"
    const v2Note =
        opts.baseFeeV2 === undefined ? "default" : "caller-supplied override"

    if (opts.assethubBaseFee || resumeAll) {
        pushFeeWrite("BridgeHubEthereumBaseFee", feeV1)
        pushFeeWrite("BridgeHubEthereumBaseFeeV2", feeV2)
        summary.push(
            `Restore AssetHub outbound base fees: V1=${feeV1} (${v1Note}), V2=${feeV2} (${v2Note}).`,
        )
    } else {
        if (opts.assethubBaseFeeV1) {
            pushFeeWrite("BridgeHubEthereumBaseFee", feeV1)
            summary.push(`Restore AssetHub V1 base fee = ${feeV1} (${v1Note}).`)
        }
        if (opts.assethubBaseFeeV2) {
            pushFeeWrite("BridgeHubEthereumBaseFeeV2", feeV2)
            summary.push(`Restore AssetHub V2 base fee = ${feeV2} (${v2Note}).`)
        }
    }

    if (bhCalls.length === 0 && ahCalls.length === 0) {
        throw new Error("buildResumeBridgePreimage: no levers selected (and `all` was not set).")
    }

    const ahCall =
        ahCalls.length === 0
            ? null
            : ahCalls.length === 1
                ? ahCalls[0]
                : assetHub.tx.utility.forceBatch(ahCalls)

    const bhXcmSend = bhCalls.length === 0
        ? null
        : await wrapBridgeHubCallsInXcmSend(assetHub, bridgeHub, bhCalls)

    let outer: SubmittableExtrinsic<"promise">
    if (bhXcmSend && ahCall) {
        outer = assetHub.tx.utility.forceBatch([bhXcmSend, ahCall])
    } else if (bhXcmSend) {
        outer = bhXcmSend
    } else if (ahCall) {
        outer = ahCall
    } else {
        throw new Error("unreachable")
    }

    const callData = outer.method.toHex()
    const hash = blake2AsHex(callData, 256)
    const encodedSize = (callData.length - 2) / 2

    const decodeUrl =
        `https://polkadot.js.org/apps/?rpc=${encodeURIComponent(assetHubWsUrl)}` +
        `#/extrinsics/decode/${callData}`

    return { hash, callData, encodedSize, decodeUrl, summary, storageWrites }
}

function isResumeAllImplied(opts: ResumeBridgeOptions): boolean {
    if (opts.all) return true
    return !(
        opts.gateway ||
        opts.gatewayV1 ||
        opts.gatewayV2 ||
        opts.inboundQueue ||
        opts.inboundQueueV1 ||
        opts.inboundQueueV2 ||
        opts.outboundQueue ||
        opts.outboundQueueV1 ||
        opts.systemFrontend ||
        opts.ethereumClient ||
        opts.assethubBaseFee ||
        opts.assethubBaseFeeV1 ||
        opts.assethubBaseFeeV2
    )
}

/**
 * Whether the option set should be treated as "halt everything",true when the
 * caller passed `all`, or when no individual flag was set (matches CLI behaviour).
 */
function isHaltAllImplied(opts: HaltBridgeOptions): boolean {
    if (opts.all) return true
    return !(
        opts.gateway ||
        opts.gatewayV1 ||
        opts.gatewayV2 ||
        opts.inboundQueue ||
        opts.inboundQueueV1 ||
        opts.inboundQueueV2 ||
        opts.outboundQueue ||
        opts.outboundQueueV1 ||
        opts.systemFrontend ||
        opts.ethereumClient ||
        opts.assethubMaxFee ||
        opts.assethubMaxFeeV1 ||
        opts.assethubMaxFeeV2
    )
}

/**
 * Build a `frame_system::set_storage` call writing `value` to the storage key
 * `twox_128(":NAME:")` on AssetHub, and return a descriptor of the write so the
 * caller can show the storage key derivation alongside the encoded preimage.
 *
 * The key derivation matches the `parameter_types! { pub storage NAME: T = ...; }`
 * macro convention used by polkadot-fellows runtimes,anyone can recompute it
 * with `xxhashAsHex(":NAME:", 128)`.
 */
function setAssetHubFeeCallWithWrite(
    assetHub: ApiPromise,
    name: string,
    value: bigint,
): { call: SubmittableExtrinsic<"promise">; write: StorageWrite } {
    const wellKnown = `:${name}:`
    const key = xxhashAsHex(wellKnown, 128, true)
    const encodedValue = u8aToHex(assetHub.createType("u128", value.toString()).toU8a())
    const call = assetHub.tx.system.setStorage([[key, encodedValue]])
    return {
        call,
        write: { name, key, value: encodedValue, sourceUrl: FEE_SOURCE_URL },
    }
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
