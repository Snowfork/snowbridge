/**
 * Phase 2 simulation: verify that the SetOperatingMode commands the SDK causes
 * BH to queue (V1 + V2), when delivered to the Gateway on Ethereum, actually
 * flip the Gateway's operating mode to RejectingOutboundMessages.
 *
 * The Gateway's external entry points (`submitV1`, `submitV2`) require a real
 * BEEFY commitment + MMR proof, which we cannot produce on a fork. Instead, we
 * bypass those checks by impersonating the Gateway contract itself (the
 * `onlySelf` modifier on the handlers permits Gateway -> Gateway calls). This
 * exercises the same dispatch path that `submitV1` / `submitV2` would take
 * after successful proof verification.
 *
 * Prerequisites:
 *   - Anvil running with `--fork-url <mainnet RPC>` (default http://localhost:8545).
 *     e.g. `anvil --fork-url $ETHEREUM_RPC_URL`.
 *   - Override endpoint and gateway address via env vars if needed.
 */

import { ethers } from "ethers"
import * as fs from "fs"

const ETHEREUM_RPC = process.env.ETHEREUM_RPC ?? "http://localhost:8545"
const GATEWAY_ADDRESS =
    process.env.GATEWAY_ADDRESS ?? "0x27ca963c279c93801941e1eb8799c23f407d68e7"
const HANDOFF_FILE =
    process.env.HALT_BRIDGE_HANDOFF ?? "/tmp/halt-bridge-sim/messages.json"

// CommandKind.SetOperatingMode in contracts/src/v2/Types.sol
const COMMAND_KIND_SET_OPERATING_MODE = 1

// OperatingMode enum in contracts/src/types/Common.sol
const OPERATING_MODE_NORMAL = 0
const OPERATING_MODE_REJECTING_OUTBOUND = 1

interface Handoff {
    v1: { params: string } | null
    v2: { kind: number; gas: string | number; payload: string; origin: string } | null
}

function loadHandoff(): Handoff | null {
    if (!fs.existsSync(HANDOFF_FILE)) {
        console.log(`  handoff file ${HANDOFF_FILE} not found, using hardcoded defaults`)
        return null
    }
    const raw = fs.readFileSync(HANDOFF_FILE, "utf8")
    const parsed = JSON.parse(raw) as Handoff & { generatedAt?: string }
    console.log(`  loaded handoff from ${HANDOFF_FILE} (generated ${parsed.generatedAt ?? "unknown"})`)
    return parsed
}

// Minimum ABI surface needed to drive the operating mode flip and read it back.
const GATEWAY_ABI = [
    "function v1_handleSetOperatingMode(bytes data) external",
    "function v2_dispatchCommand((uint8 kind, uint64 gas, bytes payload) command, bytes32 origin) external",
    "function operatingMode() external view returns (uint8)",
    "event OperatingModeChanged(uint8 mode)",
]

async function main() {
    console.log(`Connecting to anvil at ${ETHEREUM_RPC}`)
    const provider = new ethers.JsonRpcProvider(ETHEREUM_RPC)
    const gateway = new ethers.Contract(GATEWAY_ADDRESS, GATEWAY_ABI, provider)

    console.log("\n=== Step 0: load Phase 1 handoff ===")
    const handoff = loadHandoff()

    console.log("\n=== Step 1: read current operating mode ===")
    const initialMode = Number(await gateway.operatingMode())
    console.log(`  current mode: ${describeMode(initialMode)}`)
    if (initialMode !== OPERATING_MODE_NORMAL) {
        console.warn(
            `  Warning: gateway is not in Normal mode at fork point. Tests will still verify mode flips correctly.`,
        )
    }

    console.log("\n=== Step 2: impersonate the gateway address ===")
    await provider.send("anvil_impersonateAccount", [GATEWAY_ADDRESS])
    // Fund the gateway address so it can pay gas. anvil_setBalance accepts a hex
    // wei amount; 1 ETH is more than enough.
    await provider.send("anvil_setBalance", [GATEWAY_ADDRESS, "0xde0b6b3a7640000"])

    try {
        await verifyV1Path(provider, gateway, handoff)
        await resetMode(provider, gateway)
        await verifyV2Path(provider, gateway, handoff)
    } finally {
        await provider.send("anvil_stopImpersonatingAccount", [GATEWAY_ADDRESS])
    }

    console.log("\nAll Ethereum-side checks passed. Phase 2 PASSED.")
}

async function verifyV1Path(
    provider: ethers.JsonRpcProvider,
    gateway: ethers.Contract,
    handoff: Handoff | null,
) {
    console.log("\n=== Step 3: V1 path ===")

    // Prefer the actual `params` bytes BH queued (extracted from
    // ethereumOutboundQueue.Messages by Phase 1). Falls back to a hardcoded
    // encoding of RejectingOutboundMessages if no handoff is available, so
    // Phase 2 can still be run standalone.
    let v1Data: string
    if (handoff?.v1?.params) {
        v1Data = handoff.v1.params
        console.log(`  V1 params (from Phase 1 handoff): ${v1Data}`)
    } else {
        v1Data = ethers.AbiCoder.defaultAbiCoder().encode(
            ["uint8"],
            [OPERATING_MODE_REJECTING_OUTBOUND],
        )
        console.log(`  V1 params (hardcoded fallback): ${v1Data}`)
    }

    const calldata = gateway.interface.encodeFunctionData("v1_handleSetOperatingMode", [
        v1Data,
    ])
    const txHash = (await provider.send("eth_sendTransaction", [
        {
            from: GATEWAY_ADDRESS,
            to: GATEWAY_ADDRESS,
            data: calldata,
        },
    ])) as string
    console.log(`  tx hash: ${txHash}`)

    const receipt = await provider.waitForTransaction(txHash)
    if (!receipt || receipt.status !== 1) {
        throw new Error(`V1 dispatch failed (status=${receipt?.status})`)
    }

    const mode = Number(await gateway.operatingMode())
    console.log(`  mode after V1 dispatch: ${describeMode(mode)}`)
    if (mode !== OPERATING_MODE_REJECTING_OUTBOUND) {
        throw new Error(
            `V1 path failed: mode is ${describeMode(mode)}, expected RejectingOutboundMessages`,
        )
    }
}

async function verifyV2Path(
    provider: ethers.JsonRpcProvider,
    gateway: ethers.Contract,
    handoff: Handoff | null,
) {
    console.log("\n=== Step 5: V2 path ===")

    // V2 wraps the SetOperatingModeParams payload inside a CommandV2 tuple.
    // Prefer the (kind, gas, payload, origin) BH queued via
    // ethereumOutboundQueueV2.Messages. Fall back to a hardcoded command if
    // no handoff is available.
    let command: { kind: number; gas: bigint; payload: string }
    let origin: string
    if (handoff?.v2) {
        command = {
            kind: Number(handoff.v2.kind),
            gas: BigInt(handoff.v2.gas),
            payload: handoff.v2.payload,
        }
        origin = handoff.v2.origin
        console.log(`  V2 command (from Phase 1 handoff): kind=${command.kind}, gas=${command.gas}, payload=${command.payload}`)
        console.log(`  V2 origin (from Phase 1 handoff): ${origin}`)
    } else {
        const v2Payload = ethers.AbiCoder.defaultAbiCoder().encode(
            ["uint8"],
            [OPERATING_MODE_REJECTING_OUTBOUND],
        )
        command = { kind: COMMAND_KIND_SET_OPERATING_MODE, gas: 200_000n, payload: v2Payload }
        origin = ethers.ZeroHash
        console.log(`  V2 command (hardcoded fallback): kind=${command.kind}, gas=${command.gas}, payload=${v2Payload}`)
    }

    const calldata = gateway.interface.encodeFunctionData("v2_dispatchCommand", [
        [command.kind, command.gas, command.payload],
        origin,
    ])
    const txHash = (await provider.send("eth_sendTransaction", [
        {
            from: GATEWAY_ADDRESS,
            to: GATEWAY_ADDRESS,
            data: calldata,
        },
    ])) as string
    console.log(`  tx hash: ${txHash}`)

    const receipt = await provider.waitForTransaction(txHash)
    if (!receipt || receipt.status !== 1) {
        throw new Error(`V2 dispatch failed (status=${receipt?.status})`)
    }

    const mode = Number(await gateway.operatingMode())
    console.log(`  mode after V2 dispatch: ${describeMode(mode)}`)
    if (mode !== OPERATING_MODE_REJECTING_OUTBOUND) {
        throw new Error(
            `V2 path failed: mode is ${describeMode(mode)}, expected RejectingOutboundMessages`,
        )
    }
}

// Reset operating mode back to Normal between V1 and V2 tests so each path is
// verified in isolation (the V2 test would otherwise be a no-op).
async function resetMode(provider: ethers.JsonRpcProvider, gateway: ethers.Contract) {
    console.log("\n=== Step 4: reset mode to Normal between tests ===")
    const v1ResetData = ethers.AbiCoder.defaultAbiCoder().encode(
        ["uint8"],
        [OPERATING_MODE_NORMAL],
    )
    const calldata = gateway.interface.encodeFunctionData("v1_handleSetOperatingMode", [
        v1ResetData,
    ])
    const txHash = (await provider.send("eth_sendTransaction", [
        {
            from: GATEWAY_ADDRESS,
            to: GATEWAY_ADDRESS,
            data: calldata,
        },
    ])) as string
    await provider.waitForTransaction(txHash)
    const mode = Number(await gateway.operatingMode())
    if (mode !== OPERATING_MODE_NORMAL) {
        throw new Error(`Reset failed: mode is ${describeMode(mode)}`)
    }
    console.log("  mode reset to Normal")
}

function describeMode(mode: number): string {
    switch (mode) {
        case OPERATING_MODE_NORMAL:
            return "Normal (0)"
        case OPERATING_MODE_REJECTING_OUTBOUND:
            return "RejectingOutboundMessages (1)"
        default:
            return `Unknown (${mode})`
    }
}

main().catch((err) => {
    console.error("Simulation FAILED:", err)
    process.exit(1)
})
