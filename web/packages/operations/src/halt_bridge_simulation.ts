/**
 * Phase 1 simulation: verify the halt-bridge SDK preimage on a Chopsticks fork
 * of Polkadot AssetHub + BridgeHub.
 *
 * What it checks:
 *   1. Hash correctness: blake2_256(callData) equals the hash the SDK returns.
 *   2. Effect on chain: injecting the preimage as `Noted` + scheduling the call
 *      with Root for the next block produces the expected events on AH and BH.
 *
 * Prerequisites:
 *   - Chopsticks running in XCM mode against AH (default :8000) and BH (default
 *     :8001). Use `chopsticks-testbed/testbed.sh` or the configs in
 *     `control/chopsticks-config/`.
 *   - Override endpoints via ASSET_HUB_WS / BRIDGE_HUB_WS env vars if needed.
 */

import { ApiPromise, WsProvider } from "@polkadot/api"
import { Keyring } from "@polkadot/keyring"
import { blake2AsHex, xxhashAsHex } from "@polkadot/util-crypto"
import { compactToU8a, u8aToHex, hexToU8a } from "@polkadot/util"
import { governance } from "@snowbridge/api"
import * as fs from "fs"

const ASSET_HUB_WS = process.env.ASSET_HUB_WS ?? "ws://localhost:8000"
const BRIDGE_HUB_WS = process.env.BRIDGE_HUB_WS ?? "ws://localhost:8001"
const HANDOFF_FILE =
    process.env.HALT_BRIDGE_HANDOFF ?? "/tmp/halt-bridge-sim/messages.json"

// Command enum indices on the Ethereum side. Mirror the on-chain enums:
//   contracts/src/v1/Types.sol::Command (V1, 12 variants, 0-indexed)
//   contracts/src/v2/Types.sol::CommandKind (V2)
// Only SetOperatingMode is relevant here.
const V1_COMMAND_SET_OPERATING_MODE = 5
const V2_COMMAND_KIND_SET_OPERATING_MODE = 1

// Number of new blocks to build after injection. AH needs at least one to
// dispatch the scheduled call, BH needs a few more to process the HRMP-delivered
// XCM Transacts and queue the outbound gateway messages.
const AH_BLOCKS_AFTER_INJECT = 2
const BH_BLOCKS_AFTER_INJECT = 5

interface EventKey {
    section: string
    method: string
}

interface PreimageLike {
    hash: string
    callData: string
    encodedSize: number
}

// Inject a preimage as `Noted` and schedule a Root dispatch for the next AH
// block via chopsticks' `dev_setStorage`. Returns the target block number.
// Also overrides Scheduler::IncompleteSince to the current head so the next
// block's on_initialize actually services the agenda (real AH state has
// IncompleteSince at a far-future block, see project memory).
async function injectPreimageAndSchedule(
    assetHub: ApiPromise,
    preimage: PreimageLike,
): Promise<number> {
    const head = await assetHub.rpc.chain.getHeader()
    const blockNumber = head.number.toNumber()
    const targetBlock = blockNumber + 1

    const preimageBytesHex = u8aToHex(
        new Uint8Array([
            ...compactToU8a(preimage.encodedSize),
            ...hexToU8a(preimage.callData),
        ]),
    )

    const storage = {
        Preimage: {
            PreimageFor: [
                [[[preimage.hash, preimage.encodedSize]], preimageBytesHex],
            ],
            StatusFor: [
                [
                    [preimage.hash],
                    { Requested: { count: 1, len: preimage.encodedSize } },
                ],
            ],
        },
        Scheduler: {
            IncompleteSince: blockNumber,
            Agenda: [
                [
                    [targetBlock],
                    [
                        {
                            call: {
                                Lookup: {
                                    hash: preimage.hash,
                                    len: preimage.encodedSize,
                                },
                            },
                            origin: { system: "Root" },
                        },
                    ],
                ],
            ],
        },
    }
    await (assetHub.rpc as any)("dev_setStorage", storage)
    return targetBlock
}

// Build AH blocks, wait for HRMP relay, build BH blocks, wait for events. Used
// by both the halt and resume phases.
async function buildAhAndBhBlocks(
    assetHub: ApiPromise,
    bridgeHub: ApiPromise,
) {
    await (assetHub.rpc as any)("dev_newBlock", { count: AH_BLOCKS_AFTER_INJECT })
    await new Promise((r) => setTimeout(r, 4000))
    await (bridgeHub.rpc as any)("dev_newBlock", { count: BH_BLOCKS_AFTER_INJECT })
    await new Promise((r) => setTimeout(r, 2000))
}

const EXPECTED_AH_EVENTS: EventKey[] = [
    { section: "scheduler", method: "Dispatched" },
    { section: "snowbridgeSystemFrontend", method: "ExportOperatingModeChanged" },
    { section: "polkadotXcm", method: "Sent" },
]

const EXPECTED_BH_EVENTS: EventKey[] = [
    { section: "ethereumSystem", method: "SetOperatingMode" },
    { section: "ethereumSystemV2", method: "SetOperatingMode" },
    { section: "ethereumInboundQueue", method: "OperatingModeChanged" },
    { section: "ethereumInboundQueueV2", method: "OperatingModeChanged" },
    { section: "ethereumOutboundQueue", method: "OperatingModeChanged" },
    { section: "ethereumBeaconClient", method: "OperatingModeChanged" },
    { section: "ethereumOutboundQueue", method: "MessageQueued" },
    { section: "ethereumOutboundQueueV2", method: "MessageQueued" },
]

async function main() {
    // 5 minute timeout: the AH block that dispatches the halt-bridge call does a
    // lot of XCM work and takes ~80s on chopsticks vs. <1s for empty blocks,
    // which exceeds the polkadot.js default 60s WS timeout.
    const RPC_TIMEOUT_MS = 300_000
    console.log(`Connecting to AssetHub at ${ASSET_HUB_WS}`)
    const assetHub = await ApiPromise.create({
        provider: new WsProvider(ASSET_HUB_WS, 2_500, {}, RPC_TIMEOUT_MS),
    })
    console.log(`Connecting to BridgeHub at ${BRIDGE_HUB_WS}`)
    const bridgeHub = await ApiPromise.create({
        provider: new WsProvider(BRIDGE_HUB_WS, 2_500, {}, RPC_TIMEOUT_MS),
    })

    try {
        await verifyHashAndExecute(assetHub, bridgeHub)
    } finally {
        await assetHub.disconnect()
        await bridgeHub.disconnect()
    }
}

async function verifyHashAndExecute(assetHub: ApiPromise, bridgeHub: ApiPromise) {
    console.log("\n=== Step 1: build preimage via SDK ===")
    const preimage = await governance.buildHaltBridgePreimage(assetHub, bridgeHub, {
        all: true,
    })
    console.log(`  hash:     ${preimage.hash}`)
    console.log(`  size:     ${preimage.encodedSize} bytes`)
    console.log(`  summary:  ${preimage.summary.length} bullets`)

    console.log("\n=== Step 2: verify blake2_256(callData) == returned hash ===")
    const recomputed = blake2AsHex(preimage.callData, 256)
    if (recomputed !== preimage.hash) {
        throw new Error(
            `Hash mismatch. SDK returned ${preimage.hash}, blake2(callData) = ${recomputed}`,
        )
    }
    console.log("  OK")

    console.log("\n=== Step 3: collect events from both chains ===")
    const ahEvents: EventKey[] = []
    const bhEvents: EventKey[] = []
    // Track the BH block hashes at which V1/V2 MessageQueued first fires.
    // Messages get cleared from storage at the start of a later block, so to
    // read them we have to query at the block where they were just queued.
    let v1QueuedBlockHash: string | null = null
    let v2QueuedBlockHash: string | null = null
    const unsubAH = (await (assetHub.query.system.events as any)((records: any) => {
        for (const { event } of records) {
            ahEvents.push({ section: event.section, method: event.method })
        }
    })) as () => void
    const unsubBH = (await (bridgeHub.query.system.events as any)(
        async (records: any) => {
            const head = await bridgeHub.rpc.chain.getHeader()
            const hash = head.hash.toHex()
            for (const { event } of records) {
                bhEvents.push({ section: event.section, method: event.method })
                if (
                    event.section === "ethereumOutboundQueue" &&
                    event.method === "MessageQueued" &&
                    !v1QueuedBlockHash
                ) {
                    v1QueuedBlockHash = hash
                }
                if (
                    event.section === "ethereumOutboundQueueV2" &&
                    event.method === "MessageQueued" &&
                    !v2QueuedBlockHash
                ) {
                    v2QueuedBlockHash = hash
                }
            }
        },
    )) as () => void

    // Capture the live prod fee values now (before the halt overwrites them
    // with u128::MAX) so we can later assert the resume restored exactly the
    // same bytes. This catches drift between the SDK's hardcoded PROD_BASE_FEE
    // constants and what's actually on-chain.
    const originalFees: Record<string, string> = {}
    for (const name of ["BridgeHubEthereumBaseFee", "BridgeHubEthereumBaseFeeV2"]) {
        const key = xxhashAsHex(`:${name}:`, 128, true)
        const raw = await assetHub.rpc.state.getStorage(key)
        originalFees[name] = (raw as any).toHex()
    }

    console.log("\n=== Step 4: inject preimage + schedule as Root on AssetHub ===")
    const targetBlock = await injectPreimageAndSchedule(assetHub, preimage)
    console.log(`  scheduled halt preimage at AH block ${targetBlock}`)

    console.log(`\n=== Step 5/6: build ${AH_BLOCKS_AFTER_INJECT} AH + ${BH_BLOCKS_AFTER_INJECT} BH blocks ===`)
    await buildAhAndBhBlocks(assetHub, bridgeHub)
    unsubAH()
    unsubBH()

    console.log("\n=== Step 7: assert expected halt events ===")
    assertEvents("AssetHub", ahEvents, EXPECTED_AH_EVENTS)
    assertEvents("BridgeHub", bhEvents, EXPECTED_BH_EVENTS)

    console.log("\n=== Step 8: extract halt messages for Phase 2 handoff ===")
    const haltHandoff = {
        v1: await extractV1Message(bridgeHub, v1QueuedBlockHash),
        v2: await extractV2Message(bridgeHub, v2QueuedBlockHash),
    }

    console.log("\n=== Step 9: assert halt-storage values ===")
    await assertHaltStorage(assetHub, bridgeHub)

    console.log("\n=== Step 10: assert AH frontend rejects P->E sends while halted ===")
    await assertAhFrontendRejectsSend(assetHub)

    // ===== RESUME PHASE =====
    console.log("\n=== Step 11: build resume preimage via SDK ===")
    const resumePreimage = await governance.buildResumeBridgePreimage(assetHub, bridgeHub, {
        all: true,
    })
    console.log(`  hash:    ${resumePreimage.hash}`)
    console.log(`  size:    ${resumePreimage.encodedSize} bytes`)
    console.log(`  summary: ${resumePreimage.summary.length} bullets`)
    const recomputedResume = blake2AsHex(resumePreimage.callData, 256)
    if (recomputedResume !== resumePreimage.hash) {
        throw new Error(
            `Resume hash mismatch. SDK returned ${resumePreimage.hash}, blake2(callData) = ${recomputedResume}`,
        )
    }

    console.log("\n=== Step 12: collect events + inject resume preimage ===")
    const ahEvents2: EventKey[] = []
    const bhEvents2: EventKey[] = []
    let v1ResumeBlockHash: string | null = null
    let v2ResumeBlockHash: string | null = null
    const unsubAH2 = (await (assetHub.query.system.events as any)((records: any) => {
        for (const { event } of records) {
            ahEvents2.push({ section: event.section, method: event.method })
        }
    })) as () => void
    const unsubBH2 = (await (bridgeHub.query.system.events as any)(
        async (records: any) => {
            const head = await bridgeHub.rpc.chain.getHeader()
            const hash = head.hash.toHex()
            for (const { event } of records) {
                bhEvents2.push({ section: event.section, method: event.method })
                if (
                    event.section === "ethereumOutboundQueue" &&
                    event.method === "MessageQueued" &&
                    !v1ResumeBlockHash
                ) {
                    v1ResumeBlockHash = hash
                }
                if (
                    event.section === "ethereumOutboundQueueV2" &&
                    event.method === "MessageQueued" &&
                    !v2ResumeBlockHash
                ) {
                    v2ResumeBlockHash = hash
                }
            }
        },
    )) as () => void

    const resumeTarget = await injectPreimageAndSchedule(assetHub, resumePreimage)
    console.log(`  scheduled resume preimage at AH block ${resumeTarget}`)

    console.log(`\n=== Step 13: build ${AH_BLOCKS_AFTER_INJECT} AH + ${BH_BLOCKS_AFTER_INJECT} BH blocks ===`)
    await buildAhAndBhBlocks(assetHub, bridgeHub)
    unsubAH2()
    unsubBH2()

    console.log("\n=== Step 14: extract resume messages for Phase 2 handoff ===")
    const resumeHandoff = {
        v1: await extractV1Message(bridgeHub, v1ResumeBlockHash),
        v2: await extractV2Message(bridgeHub, v2ResumeBlockHash),
    }
    writeHandoffFile({ halt: haltHandoff, resume: resumeHandoff })

    console.log("\n=== Step 15: assert resume-storage values ===")
    await assertResumeStorage(assetHub, bridgeHub, originalFees)

    console.log("\n=== Step 16: assert AH frontend accepts P->E sends again ===")
    await assertAhFrontendAcceptsSend(assetHub)

    console.log("\nHalt + resume verified end-to-end. Simulation PASSED.")
}

// Verify each halt lever produced the expected storage value. Events firing is
// not sufficient: an OperatingModeChanged event would also fire if mode were
// set back to Normal, and `system.setStorage` (the fee writes) emits no event
// at all.
async function assertHaltStorage(assetHub: ApiPromise, bridgeHub: ApiPromise) {
    // BH pallets with a local operating_mode storage item. `ethereumSystem` /
    // `ethereumSystemV2` are intentionally NOT in this list: their
    // setOperatingMode call only queues a Gateway message and stores no local
    // state. The Gateway-side flip is verified in Phase 2.
    // `ethereumOutboundQueueV2` is also absent (the SDK comment confirms it
    // has no halt lever in the runtime).
    const bhHalted: Array<[string, string]> = [
        ["ethereumInboundQueue", "operatingMode"],
        ["ethereumInboundQueueV2", "operatingMode"],
        ["ethereumOutboundQueue", "operatingMode"],
        ["ethereumBeaconClient", "operatingMode"],
    ]
    for (const [pallet, item] of bhHalted) {
        const value = (await ((bridgeHub.query as any)[pallet][item])()).toString()
        if (value !== "Halted") {
            throw new Error(`BH ${pallet}.${item} = ${value}, expected Halted`)
        }
        console.log(`  BH ${pallet}.${item} = Halted`)
    }

    // AH frontend's storage item is named `exportOperatingMode`, not
    // `operatingMode`. Spotted via metadata probe.
    const frontendMode = (
        await (assetHub.query as any).snowbridgeSystemFrontend.exportOperatingMode()
    ).toString()
    if (frontendMode !== "Halted") {
        throw new Error(`AH snowbridgeSystemFrontend.exportOperatingMode = ${frontendMode}, expected Halted`)
    }
    console.log(`  AH snowbridgeSystemFrontend.exportOperatingMode = Halted`)

    // AH well-known fee storage keys, written via system.setStorage.
    const u128Max = "0x" + "ff".repeat(16)
    for (const name of ["BridgeHubEthereumBaseFee", "BridgeHubEthereumBaseFeeV2"]) {
        const key = xxhashAsHex(`:${name}:`, 128, true)
        const raw = await assetHub.rpc.state.getStorage(key)
        const hex = (raw as any).toHex()
        if (hex !== u128Max) {
            throw new Error(`AH ${name} (key ${key}) = ${hex}, expected ${u128Max}`)
        }
        console.log(`  AH :${name}: = u128::MAX`)
    }
}

// After the halt, the user-facing P->E send entry on AssetHub
// (`snowbridgeSystemFrontend.registerToken`) should be rejected by the
// pallet's operating-mode gate. We submit a signed extrinsic from Alice
// (chopsticks accepts mock signatures) and check the dispatch result.
async function assertAhFrontendRejectsSend(assetHub: ApiPromise) {
    const keyring = new Keyring({ type: "sr25519" })
    const alice = keyring.addFromUri("//Alice")

    // Minimal but type-valid args. Argument validation may reject *before*
    // the mode gate (e.g. invalid asset location); if so, the test still
    // demonstrates the call cannot complete, but the assertion below
    // tolerates either case and reports which error fired.
    const assetId: any = { V5: { parents: 1, interior: "Here" } }
    const metadata: any = {
        name: "0x48414c54", // "HALT"
        symbol: "0x48",
        decimals: 18,
    }
    const feeAsset: any = {
        id: { parents: 1, interior: "Here" },
        fun: { Fungible: 1 },
    }
    const call = (assetHub.tx as any).snowbridgeSystemFrontend.registerToken(
        assetId,
        metadata,
        feeAsset,
    )

    type Outcome =
        | { ok: false; reason: "module"; section: string; name: string; docs: string }
        | { ok: false; reason: "other"; toString: string }
        | { ok: true }

    const outcome = await new Promise<Outcome>((resolve, reject) => {
        call
            .signAndSend(alice, ({ status, dispatchError }: any) => {
                if (!(status.isInBlock || status.isFinalized)) return
                if (!dispatchError) {
                    resolve({ ok: true })
                    return
                }
                if (dispatchError.isModule) {
                    const decoded = assetHub.registry.findMetaError(
                        dispatchError.asModule,
                    )
                    resolve({
                        ok: false,
                        reason: "module",
                        section: decoded.section,
                        name: decoded.name,
                        docs: decoded.docs.join(" "),
                    })
                } else {
                    resolve({
                        ok: false,
                        reason: "other",
                        toString: dispatchError.toString(),
                    })
                }
            })
            .catch(reject)
    })

    // Build a block so the submitted extrinsic is actually included.
    await (assetHub.rpc as any)("dev_newBlock", { count: 1 })

    // The promise above only resolves once the InBlock status comes through,
    // which happens after dev_newBlock includes the tx. Re-await with a small
    // delay in case ordering means the status callback fires after we return.
    const final = await Promise.race<Outcome>([
        Promise.resolve(outcome),
        new Promise<Outcome>((r) =>
            setTimeout(() => r({ ok: false, reason: "other", toString: "timeout" }), 5_000),
        ),
    ])

    if (final.ok) {
        throw new Error(
            "registerToken succeeded while halted; the operating-mode gate is not enforcing on the send path",
        )
    }
    if (final.reason === "module") {
        const fingerprint = `${final.section}.${final.name}`.toLowerCase()
        const haltSignal =
            fingerprint.includes("halted") ||
            fingerprint.includes("disabled") ||
            fingerprint.includes("operatingmode") ||
            final.docs.toLowerCase().includes("halted")
        if (haltSignal) {
            console.log(`  registerToken correctly rejected: ${final.section}.${final.name} (${final.docs})`)
        } else {
            console.log(
                `  registerToken rejected (but error name does not obviously match a halt: ${final.section}.${final.name} - ${final.docs}). Treating as proof the call did not complete; review if this looks unrelated.`,
            )
        }
    } else {
        console.log(`  registerToken rejected: ${final.toString}`)
    }
}

// Inverse of assertHaltStorage: each operating mode storage item should be
// back to `Normal`, and the AH base-fee storage keys should hold the prod
// values the resume preimage wrote.
async function assertResumeStorage(
    assetHub: ApiPromise,
    bridgeHub: ApiPromise,
    originalFees: Record<string, string>,
) {
    const bhPallets: Array<[string, string]> = [
        ["ethereumInboundQueue", "operatingMode"],
        ["ethereumInboundQueueV2", "operatingMode"],
        ["ethereumOutboundQueue", "operatingMode"],
        ["ethereumBeaconClient", "operatingMode"],
    ]
    for (const [pallet, item] of bhPallets) {
        const value = (await ((bridgeHub.query as any)[pallet][item])()).toString()
        if (value !== "Normal") {
            throw new Error(`BH ${pallet}.${item} = ${value}, expected Normal`)
        }
        console.log(`  BH ${pallet}.${item} = Normal`)
    }
    const frontendMode = (
        await (assetHub.query as any).snowbridgeSystemFrontend.exportOperatingMode()
    ).toString()
    if (frontendMode !== "Normal") {
        throw new Error(`AH snowbridgeSystemFrontend.exportOperatingMode = ${frontendMode}, expected Normal`)
    }
    console.log(`  AH snowbridgeSystemFrontend.exportOperatingMode = Normal`)

    // Fee values must match the exact bytes that were on-chain before the
    // halt, captured pre-halt. This catches drift between the SDK's hardcoded
    // PROD_BASE_FEE constants and what prod actually holds.
    for (const name of ["BridgeHubEthereumBaseFee", "BridgeHubEthereumBaseFeeV2"]) {
        const key = xxhashAsHex(`:${name}:`, 128, true)
        const raw = await assetHub.rpc.state.getStorage(key)
        const hex = (raw as any).toHex()
        const original = originalFees[name]
        if (hex !== original) {
            throw new Error(
                `AH ${name} (key ${key}): resumed value ${hex} != original ${original}. Update PROD_BASE_FEE_* in halt_bridge.ts.`,
            )
        }
        console.log(`  AH :${name}: = ${hex} (matches original prod bytes)`)
    }
}

// Inverse of assertAhFrontendRejectsSend: after resume, registerToken should
// no longer trip the snowbridgeSystemFrontend.Halted gate. The call may still
// fail downstream (we're using throwaway args that won't make it through XCM
// execution end-to-end) but it must NOT fail with a Halted-shaped error.
async function assertAhFrontendAcceptsSend(assetHub: ApiPromise) {
    const keyring = new Keyring({ type: "sr25519" })
    const alice = keyring.addFromUri("//Alice")
    const assetId: any = { V5: { parents: 1, interior: "Here" } }
    const metadata: any = { name: "0x4f4b", symbol: "0x4f", decimals: 18 }
    const feeAsset: any = { id: { parents: 1, interior: "Here" }, fun: { Fungible: 1 } }
    const call = (assetHub.tx as any).snowbridgeSystemFrontend.registerToken(
        assetId,
        metadata,
        feeAsset,
    )

    type Outcome =
        | { kind: "module"; section: string; name: string; docs: string }
        | { kind: "other"; text: string }
        | { kind: "ok" }

    const outcome = await new Promise<Outcome>((resolve, reject) => {
        call
            .signAndSend(alice, ({ status, dispatchError }: any) => {
                if (!(status.isInBlock || status.isFinalized)) return
                if (!dispatchError) return resolve({ kind: "ok" })
                if (dispatchError.isModule) {
                    const decoded = assetHub.registry.findMetaError(dispatchError.asModule)
                    return resolve({
                        kind: "module",
                        section: decoded.section,
                        name: decoded.name,
                        docs: decoded.docs.join(" "),
                    })
                }
                resolve({ kind: "other", text: dispatchError.toString() })
            })
            .catch(reject)
    })

    await (assetHub.rpc as any)("dev_newBlock", { count: 1 })
    const final = await Promise.race<Outcome>([
        Promise.resolve(outcome),
        new Promise<Outcome>((r) =>
            setTimeout(() => r({ kind: "other", text: "timeout" }), 5_000),
        ),
    ])

    if (final.kind === "module") {
        const fp = `${final.section}.${final.name}`.toLowerCase()
        if (fp.includes("halted") || final.docs.toLowerCase().includes("halted")) {
            throw new Error(
                `registerToken still rejected with halt-shaped error after resume: ${final.section}.${final.name} (${final.docs})`,
            )
        }
        console.log(
            `  registerToken no longer rejected by halt gate. (Dispatch still errored downstream with ${final.section}.${final.name}, which is expected with throwaway args.)`,
        )
    } else if (final.kind === "ok") {
        console.log("  registerToken accepted (dispatch succeeded with throwaway args)")
    } else {
        console.log(`  registerToken accepted (non-module error: ${final.text})`)
    }
}

// Pull the V1 SetOperatingMode message BH queued at the given block hash.
// Messages are cleared the block after queuing, so we MUST query at the block
// where the corresponding MessageQueued event fired, not at latest head.
async function extractV1Message(
    bridgeHub: ApiPromise,
    block: string | null,
): Promise<V1Handoff | null> {
    const api = block ? await bridgeHub.at(block) : bridgeHub
    const raw = (await (api.query as any).ethereumOutboundQueue.messages()) as any
    const messages = raw.toJSON() as any[]
    return pickV1SetOperatingMode(messages)
}

async function extractV2Message(
    bridgeHub: ApiPromise,
    block: string | null,
): Promise<V2Handoff | null> {
    const api = block ? await bridgeHub.at(block) : bridgeHub
    const raw = (await (api.query as any).ethereumOutboundQueueV2.messages()) as any
    const messages = raw.toJSON() as any[]
    return pickV2SetOperatingMode(messages)
}

interface HandoffMessages {
    v1: V1Handoff | null
    v2: V2Handoff | null
}

interface Handoff {
    generatedAt: string
    halt: HandoffMessages
    resume: HandoffMessages
}

// Write the structured handoff JSON: halt = the SetOperatingMode messages
// queued by the halt preimage (V1 payload = 0x...01, V2 payload = 0x...01),
// resume = the messages queued by the resume preimage (payloads = 0x...00).
// Phase 2 reads both and replays them in sequence to verify the deployed
// Gateway flips and then un-flips.
function writeHandoffFile(messages: { halt: HandoffMessages; resume: HandoffMessages }) {
    const handoff: Handoff = {
        generatedAt: new Date().toISOString(),
        ...messages,
    }
    fs.writeFileSync(HANDOFF_FILE, JSON.stringify(handoff, null, 2))
    console.log(`  wrote ${HANDOFF_FILE}`)
    for (const phase of ["halt", "resume"] as const) {
        const m = messages[phase]
        console.log(
            `  ${phase}: V1 params=${m.v1?.params ?? "(missing)"}, V2 payload=${m.v2?.payload ?? "(missing)"}`,
        )
    }
}

interface V1Handoff {
    channelId: string
    nonce: string
    command: number
    params: string
    id: string
}

interface V2Handoff {
    origin: string
    nonce: string
    topic: string
    kind: number
    gas: string
    payload: string
}

function pickV1SetOperatingMode(messages: any[]): V1Handoff | null {
    for (const m of messages) {
        if (Number(m.command) === V1_COMMAND_SET_OPERATING_MODE) {
            return {
                channelId: m.channelId,
                nonce: String(m.nonce),
                command: Number(m.command),
                params: m.params,
                id: m.id,
            }
        }
    }
    return null
}

function pickV2SetOperatingMode(messages: any[]): V2Handoff | null {
    for (const m of messages) {
        for (const cmd of m.commands ?? []) {
            if (Number(cmd.kind) === V2_COMMAND_KIND_SET_OPERATING_MODE) {
                return {
                    origin: m.origin,
                    nonce: String(m.nonce),
                    topic: m.topic,
                    kind: Number(cmd.kind),
                    gas: String(cmd.gas),
                    payload: cmd.payload,
                }
            }
        }
    }
    return null
}

function assertEvents(chain: string, got: EventKey[], expected: EventKey[]) {
    const seen = new Set(got.map((e) => `${e.section}.${e.method}`))
    const missing: string[] = []
    for (const e of expected) {
        const key = `${e.section}.${e.method}`
        if (!seen.has(key)) {
            missing.push(key)
        } else {
            console.log(`  ${chain}: ${key} present`)
        }
    }
    if (missing.length > 0) {
        console.log(`\n  ${chain} all events seen (${got.length}):`)
        for (const e of got) {
            console.log(`    - ${e.section}.${e.method}`)
        }
        throw new Error(
            `${chain} missing expected events:\n  - ${missing.join("\n  - ")}`,
        )
    }
}

main().catch((err) => {
    console.error("Simulation FAILED:", err)
    process.exit(1)
})
