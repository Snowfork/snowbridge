import { ApiPromise } from "@polkadot/api"
import { SubmittableExtrinsic } from "@polkadot/api/types"
import { blake2AsHex } from "@polkadot/util-crypto"
import { hexToU8a, u8aToHex } from "@polkadot/util"

import { HaltBridgePreimage } from "./halt_bridge"

// Default WS endpoints embedded in the emitted papi.how URLs. These match what
// joepetrowski/opengov-cli prints, so the UI links open against the same
// metadata source opengov-cli would have used.
const DEFAULT_AH_WS = "wss://asset-hub-polkadot-rpc.dwellir.com"
const DEFAULT_COLLECTIVES_WS = "wss://polkadot-collectives-rpc.polkadot.io"

// papi.how host. opengov-cli emits dev.papi.how URLs, match for parity.
const DEFAULT_PAPI_HOW_BASE = "https://dev.papi.how"

// Asset Hub para ID, used as the XCM dest from Collectives. AH is where
// pallet_whitelist and pallet_referenda live post-AHM.
const ASSET_HUB_POLKADOT_ID = 1000

// opengov-cli hardcodes After(10) for the Fellowship-side enactment.
const FELLOWSHIP_ENACTMENT_AFTER_BLOCKS = 10

// papi.how networkId values, matching opengov-cli's output.
const PAPI_HOW_NETWORK_ASSET_HUB = "polkadot_asset_hub"
const PAPI_HOW_NETWORK_COLLECTIVES = "polkadot_collectives"

export interface SubmissionUrls {
    /**
     * Asset Hub batch URL: `utility.forceBatch([preimage.notePreimage(wrapped),
     * referenda.submit(Origins(WhitelistedCaller), Lookup{hash, len}, After(n))])`.
     * Anyone on the operator team can submit this.
     */
    assetHubBatchUrl: string
    /**
     * Hex bytes of the Asset Hub batch call (the `data=` payload of {@link assetHubBatchUrl}),
     * exposed for parity testing against opengov-cli.
     */
    assetHubBatchCallData: string
    /**
     * Collectives Chain URL: `utility.forceBatch([fellowshipReferenda.submit(
     * Origins(FellowshipOrigins::Fellows), Inline(xcm_send), After(10))])` where
     * `xcm_send` is a `polkadotXcm.send` to Asset Hub carrying a Transact of
     * `whitelist.whitelistCall(preimageHash)`. **Must be submitted by a Fellow
     * of rank 3 or higher.**
     */
    fellowshipWhitelistUrl: string
    /**
     * Hex bytes of the Collectives call (the `data=` payload of {@link fellowshipWhitelistUrl}),
     * exposed for parity testing against opengov-cli.
     */
    fellowshipWhitelistCallData: string
    /**
     * Hash of the user-supplied preimage call (the input to this function).
     * This is the hash that goes inside `whitelist.whitelistCall` on the
     * Fellowship side. **Not** the hash referenced by the AH public referendum
     * (that one hashes the wrapped `dispatchWhitelistedCallWithPreimage` call).
     */
    preimageHash: string
    /**
     * Hash of the wrapped preimage (`whitelist.dispatchWhitelistedCallWithPreimage(call)`).
     * This is what `preimage.notePreimage` stores on AH and what the AH public
     * referendum's `Lookup` references.
     */
    wrappedPreimageHash: string
    /**
     * Length in bytes of the wrapped preimage, the `len` field of the AH
     * referendum's `Lookup`.
     */
    wrappedPreimageLen: number
}

export interface SubmissionOptions {
    /**
     * Enactment delay (in blocks) for the AH public WhitelistedCaller
     * referendum. Defaults to 10, matching opengov-cli's default. The
     * Fellowship-side enactment is always hardcoded to `After(10)` to match
     * opengov-cli.
     */
    enactmentAfterBlocks?: number
    /** Override the papi.how host. Defaults to dev.papi.how (opengov-cli parity). */
    papiHowBase?: string
    /** Override the AH WS URL embedded in the papi.how URL. */
    assetHubWsUrl?: string
    /** Override the Collectives WS URL embedded in the papi.how URL. */
    collectivesWsUrl?: string
}

/**
 * Build the two submission URLs a Snowbridge halt preimage needs to land on
 * the Polkadot WhitelistedCaller track:
 *
 *  1. An Asset Hub URL the operator submits to note the (wrapped) preimage
 *     and open the public Whitelisted Caller referendum.
 *  2. A Collectives Chain URL that a rank-3+ Fellow submits to open the
 *     Fellowship whitelist referendum, which XCM-Transacts into AH to call
 *     `whitelist.whitelistCall(preimageHash)`.
 *
 * Mirrors joepetrowski/opengov-cli's `polkadot_fellowship_referenda` flow
 * (src/submit_referendum.rs ~L628-L811). The encoded call bytes are byte-for-
 * byte identical to what opengov-cli emits for the same preimage (verified by
 * the `opengov_submission_check` script in @snowbridge/operations).
 *
 * @param assetHub     Connected ApiPromise for Asset Hub Polkadot (provides
 *                     metadata for `whitelist`, `preimage`, `referenda`,
 *                     `utility`, `Origins`).
 * @param collectives  Connected ApiPromise for Polkadot Collectives Chain
 *                     (provides metadata for `fellowshipReferenda`, `polkadotXcm`,
 *                     `utility`, `FellowshipOrigins`).
 * @param preimage     The halt-bridge preimage to wrap, either a {@link HaltBridgePreimage}
 *                     (use its `callData` field) or a 0x-prefixed hex string of
 *                     the raw preimage call bytes.
 * @param opts         Optional URL/enactment overrides.
 */
export async function buildHaltBridgeSubmissionUrls(
    assetHub: ApiPromise,
    collectives: ApiPromise,
    preimage: HaltBridgePreimage | string,
    opts: SubmissionOptions = {},
): Promise<SubmissionUrls> {
    const userCallHex =
        typeof preimage === "string" ? preimage : preimage.callData

    const assetHubWsUrl = opts.assetHubWsUrl ?? DEFAULT_AH_WS
    const collectivesWsUrl = opts.collectivesWsUrl ?? DEFAULT_COLLECTIVES_WS
    const papiHowBase = opts.papiHowBase ?? DEFAULT_PAPI_HOW_BASE
    const ahEnactmentAfter =
        opts.enactmentAfterBlocks ?? FELLOWSHIP_ENACTMENT_AFTER_BLOCKS

    const userCallBytes = hexToU8a(userCallHex)
    const preimageHash = blake2AsHex(userCallBytes, 256)

    // Wrap the user call in whitelist.dispatchWhitelistedCallWithPreimage.
    // The wrapped bytes are what gets noted on AH and referenced by the
    // public referendum; the wrapped hash is distinct from the raw preimage
    // hash that the Fellowship whitelist call hashes.
    const userCallDecoded = assetHub.createType("Call", userCallBytes)
    const wrappedCall = assetHub.tx.whitelist.dispatchWhitelistedCallWithPreimage(
        userCallDecoded,
    )
    const wrappedCallBytes = wrappedCall.method.toU8a()
    const wrappedPreimageHash = blake2AsHex(wrappedCallBytes, 256)
    const wrappedPreimageLen = wrappedCallBytes.length

    const assetHubBatch = buildAssetHubBatch(
        assetHub,
        wrappedCallBytes,
        wrappedPreimageHash,
        wrappedPreimageLen,
        ahEnactmentAfter,
    )
    const assetHubBatchCallData = u8aToHex(assetHubBatch.method.toU8a())

    const collectivesBatch = buildCollectivesBatch(
        assetHub,
        collectives,
        preimageHash,
    )
    const fellowshipWhitelistCallData = u8aToHex(collectivesBatch.method.toU8a())

    return {
        assetHubBatchUrl: buildPapiHowUrl(
            papiHowBase,
            PAPI_HOW_NETWORK_ASSET_HUB,
            assetHubWsUrl,
            assetHubBatchCallData,
        ),
        assetHubBatchCallData,
        fellowshipWhitelistUrl: buildPapiHowUrl(
            papiHowBase,
            PAPI_HOW_NETWORK_COLLECTIVES,
            collectivesWsUrl,
            fellowshipWhitelistCallData,
        ),
        fellowshipWhitelistCallData,
        preimageHash,
        wrappedPreimageHash,
        wrappedPreimageLen,
    }
}

/**
 * Convenience wrapper for the resume case. The wire format is identical
 * (resume preimages go through the same WhitelistedCaller track), so this
 * just delegates to {@link buildHaltBridgeSubmissionUrls}.
 */
export async function buildResumeBridgeSubmissionUrls(
    assetHub: ApiPromise,
    collectives: ApiPromise,
    preimage: HaltBridgePreimage | string,
    opts: SubmissionOptions = {},
): Promise<SubmissionUrls> {
    return buildHaltBridgeSubmissionUrls(assetHub, collectives, preimage, opts)
}

function buildAssetHubBatch(
    assetHub: ApiPromise,
    wrappedCallBytes: Uint8Array,
    wrappedPreimageHash: string,
    wrappedPreimageLen: number,
    enactmentAfterBlocks: number,
): SubmittableExtrinsic<"promise"> {
    const notePreimage = assetHub.tx.preimage.notePreimage(
        u8aToHex(wrappedCallBytes),
    )

    // Plain-object args; @polkadot/api resolves the runtime-specific
    // OriginCaller and Bounded enums from referenda.submit's metadata.
    const submit = assetHub.tx.referenda.submit(
        { Origins: "WhitelistedCaller" } as any,
        { Lookup: { hash: wrappedPreimageHash, len: wrappedPreimageLen } } as any,
        { After: enactmentAfterBlocks } as any,
    )

    return assetHub.tx.utility.forceBatch([notePreimage, submit])
}

function buildCollectivesBatch(
    assetHub: ApiPromise,
    collectives: ApiPromise,
    rawPreimageHash: string,
): SubmittableExtrinsic<"promise"> {
    // Inner AH call that the Fellowship XCM Transacts into:
    //   whitelist.whitelistCall(preimageHash)
    const whitelistCall = assetHub.tx.whitelist.whitelistCall(rawPreimageHash)
    const whitelistCallBytes = whitelistCall.method.toU8a()

    // XCM message: UnpaidExecution + Transact(whitelistCall). No fee asset
    // because AH grants free execution to sibling Collectives via the
    // UnpaidExecution barrier.
    const message = {
        V5: [
            {
                UnpaidExecution: {
                    weightLimit: "Unlimited",
                    checkOrigin: null,
                },
            },
            {
                Transact: {
                    originKind: "Xcm",
                    fallbackMaxWeight: null,
                    call: { encoded: u8aToHex(whitelistCallBytes) },
                },
            },
        ],
    }

    // Dest from Collectives to AH: { parents: 1, interior: X1([Parachain(1000)]) }.
    const dest = {
        V5: {
            parents: 1,
            interior: { X1: [{ Parachain: ASSET_HUB_POLKADOT_ID }] },
        },
    }

    const xcmSend = collectives.tx.polkadotXcm.send(dest, message)
    const xcmSendBytes = xcmSend.method.toU8a()

    // fellowshipReferenda.submit with FellowshipOrigins::Fellows origin and
    // Inline(xcm_send) proposal. opengov-cli only uses Inline when bytes
    // fit in the 128-byte BoundedVec; the whitelist-call wrapper is small
    // enough that this always holds.
    if (xcmSendBytes.length > 128) {
        throw new Error(
            `buildCollectivesBatch: XCM send bytes (${xcmSendBytes.length}) exceed 128-byte Inline cap; ` +
                "would need a separate Preimage::notePreimage on Collectives. Not implemented.",
        )
    }

    const submit = collectives.tx.fellowshipReferenda.submit(
        { FellowshipOrigins: "Fellows" } as any,
        { Inline: u8aToHex(xcmSendBytes) } as any,
        { After: FELLOWSHIP_ENACTMENT_AFTER_BLOCKS } as any,
    )

    // Even for a single call, opengov-cli wraps in force_batch for output
    // consistency; match it byte-for-byte.
    return collectives.tx.utility.forceBatch([submit])
}

function buildPapiHowUrl(
    base: string,
    networkId: string,
    wsUrl: string,
    callData: string,
): string {
    return (
        `${base}/extrinsics#data=${callData}` +
        `&networkId=${networkId}` +
        `&endpoint=${encodeURIComponent(wsUrl)}`
    )
}
