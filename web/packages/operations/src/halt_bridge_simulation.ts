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

    console.log("\n=== Step 4: inject preimage + schedule as Root on AssetHub ===")
    const head = await assetHub.rpc.chain.getHeader()
    const blockNumber = head.number.toNumber()
    const targetBlock = blockNumber + 1
    console.log(`  current AH block: ${blockNumber}, scheduling at ${targetBlock}`)

    // `Preimage::PreimageFor` is stored as `Bytes` (SCALE Vec<u8>), so the
    // raw call bytes must be prefixed with a SCALE-compact length. The SDK's
    // `callData` is the bare call hex (no prefix), the same form blake2 is
    // computed over.
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
                    {
                        Requested: {
                            count: 1,
                            len: preimage.encodedSize,
                        },
                    },
                ],
            ],
        },
        Scheduler: {
            // Override IncompleteSince to the current head. Real AH state has
            // this at a far-future block (~31M), which makes
            // pallet_scheduler::service_agendas use that as `when` and skip
            // our current-block agenda because `when <= now` never holds.
            // Pointing it at `blockNumber` (current head) lets the loop run
            // for `when = now = targetBlock` in the next block's on_initialize.
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

    // Diagnostic: confirm agenda + preimage landed correctly.
    const agendaBefore = await (assetHub.query as any).scheduler.agenda(targetBlock)
    console.log(`  agenda[${targetBlock}] right after setStorage:`, JSON.stringify(agendaBefore.toHuman()))
    try {
        const pf = await (assetHub.query as any).preimage.preimageFor([
            preimage.hash,
            preimage.encodedSize,
        ])
        console.log(`  preimageFor decode OK, length=${pf.toHex().length / 2 - 1} bytes`)
    } catch (e: any) {
        console.log(`  preimageFor decode FAILED:`, e.message?.slice(0, 200))
    }

    console.log(`\n=== Step 5: build ${AH_BLOCKS_AFTER_INJECT} new AH blocks ===`)
    await (assetHub.rpc as any)("dev_newBlock", { count: AH_BLOCKS_AFTER_INJECT })

    const agendaAfter = await (assetHub.query as any).scheduler.agenda(targetBlock)
    console.log(`  agenda[${targetBlock}] after dev_newBlock:`, JSON.stringify(agendaAfter.toHuman()))
    const newHead = await assetHub.rpc.chain.getHeader()
    console.log(`  new AH head: ${newHead.number.toNumber()}`)

    console.log("  waiting 4s for HRMP relay")
    await new Promise((r) => setTimeout(r, 4000))

    console.log(`\n=== Step 6: build ${BH_BLOCKS_AFTER_INJECT} new BH blocks ===`)
    await (bridgeHub.rpc as any)("dev_newBlock", { count: BH_BLOCKS_AFTER_INJECT })

    console.log("  waiting 2s for events to settle")
    await new Promise((r) => setTimeout(r, 2000))

    unsubAH()
    unsubBH()

    console.log("\n=== Step 7: assert expected events ===")
    assertEvents("AssetHub", ahEvents, EXPECTED_AH_EVENTS)
    assertEvents("BridgeHub", bhEvents, EXPECTED_BH_EVENTS)

    console.log("\n=== Step 8: extract queued messages for Phase 2 handoff ===")
    await writeHandoffFile(bridgeHub, v1QueuedBlockHash, v2QueuedBlockHash)

    console.log("\n=== Step 9: assert halt-storage values ===")
    await assertHaltStorage(assetHub, bridgeHub)

    console.log("\nAll expected events present + storage values correct. Simulation PASSED.")
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

// Extract the V1 + V2 SetOperatingMode messages BH queued and write them to a
// JSON handoff file. Phase 2 reads this and replays the actual bytes against
// the Gateway, so V1 params + V2 command struct come from real on-chain state
// instead of being hardcoded.
async function writeHandoffFile(
    bridgeHub: ApiPromise,
    v1Block: string | null,
    v2Block: string | null,
) {
    const v1Api = v1Block ? await bridgeHub.at(v1Block) : bridgeHub
    const v2Api = v2Block ? await bridgeHub.at(v2Block) : bridgeHub
    const v1Raw = (await (v1Api.query as any).ethereumOutboundQueue.messages()) as any
    const v2Raw = (await (v2Api.query as any).ethereumOutboundQueueV2.messages()) as any
    const v1Messages = v1Raw.toJSON() as any[]
    const v2Messages = v2Raw.toJSON() as any[]
    console.log(
        `  V1 messages@${v1Block ?? "latest"}: ${v1Messages.length}, V2 messages@${v2Block ?? "latest"}: ${v2Messages.length}`,
    )

    const v1 = pickV1SetOperatingMode(v1Messages)
    const v2 = pickV2SetOperatingMode(v2Messages)

    const handoff = {
        generatedAt: new Date().toISOString(),
        v1,
        v2,
    }
    fs.writeFileSync(HANDOFF_FILE, JSON.stringify(handoff, null, 2))
    console.log(`  wrote ${HANDOFF_FILE}`)
    console.log(`  V1 params: ${v1?.params ?? "(missing)"}`)
    console.log(`  V2 command: ${v2 ? `kind=${v2.kind} gas=${v2.gas} payload=${v2.payload}` : "(missing)"}`)
    console.log(`  V2 origin: ${v2?.origin ?? "(missing)"}`)
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
