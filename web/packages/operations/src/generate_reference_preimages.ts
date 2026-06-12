/**
 * Generate the canonical halt / resume preimage reference.
 *
 * This is the trust anchor for the "verify the preimage" flow discussed with the
 * Polkadot Fellowship: the JSON it emits is committed to
 * polkadot-ecosystem-tests (PET), where a Chopsticks test executes the exact
 * committed `callData` against forked live Asset Hub + Bridge Hub every ~6 hours
 * and asserts the bridge actually halts. During an incident, an operator (or a
 * whitelisting Fellow) compares the `hash` shown by the governance tool against
 * the `hash` in PET `master`, a single string comparison instead of decoding
 * SCALE bytes under pressure.
 *
 * The preimage is built from `governance.FULL_HALT_OPTIONS` /
 * `governance.FULL_RESUME_OPTIONS`, the same canonical "everything" option sets
 * the frontend's full-halt / full-resume default uses, so the committed bytes
 * are byte-identical to what an operator generates. The BridgeHub Transacts now
 * carry a constant fallback weight (no live weight query), so the bytes are
 * deterministic for a given runtime and only change on a genuine semantic change
 * (pallet index, call encoding, XCM version, or the prod resume fees).
 *
 * Usage:
 *   npx ts-node src/generate_reference_preimages.ts            # print JSON to stdout
 *   npx ts-node src/generate_reference_preimages.ts --write    # also write the file below
 *
 * Env overrides: ASSET_HUB_WS, BRIDGE_HUB_WS.
 */

import { ApiPromise, WsProvider } from "@polkadot/api"
import { governance } from "@snowbridge/api"
import { blake2AsHex } from "@polkadot/util-crypto"
import { writeFileSync, mkdirSync } from "fs"
import { dirname, join } from "path"

const ASSET_HUB_WS = process.env.ASSET_HUB_WS ?? "wss://polkadot-asset-hub-rpc.polkadot.io"
const BRIDGE_HUB_WS = process.env.BRIDGE_HUB_WS ?? "wss://polkadot-bridge-hub-rpc.polkadot.io"

// Transient (gitignored) output path. This file is NOT committed: PET owns the
// canonical reference. Copy this JSON (and the decode links printed to stderr)
// into the PET PR at packages/shared/src/snowbridge/referencePreimages.json.
const OUTPUT_PATH = join(__dirname, "..", "reference", "halt_reference_preimages.json")

interface RuntimeInfo {
    specName: string
    specVersion: number
}

interface PreimageEntry {
    hash: string
    callData: string
    encodedSize: number
}

interface ReferenceFile {
    /** Human note: this file is the canonical, Fellowship-reviewed preimage set. */
    description: string
    generatedAt: string
    assetHubRuntime: RuntimeInfo
    bridgeHubRuntime: RuntimeInfo
    halt: PreimageEntry
    resume: PreimageEntry
}

function runtimeInfo(api: ApiPromise): RuntimeInfo {
    return {
        specName: api.runtimeVersion.specName.toString(),
        specVersion: api.runtimeVersion.specVersion.toNumber(),
    }
}

async function main() {
    const write = process.argv.includes("--write")

    console.error(`Connecting to AssetHub:  ${ASSET_HUB_WS}`)
    console.error(`Connecting to BridgeHub: ${BRIDGE_HUB_WS}`)
    const [assetHub, bridgeHub] = await Promise.all([
        ApiPromise.create({ provider: new WsProvider(ASSET_HUB_WS) }),
        ApiPromise.create({ provider: new WsProvider(BRIDGE_HUB_WS) }),
    ])

    try {
        const halt = await governance.buildHaltBridgePreimage(
            assetHub,
            bridgeHub,
            governance.FULL_HALT_OPTIONS,
        )
        const resume = await governance.buildResumeBridgePreimage(
            assetHub,
            bridgeHub,
            governance.FULL_RESUME_OPTIONS,
        )

        // Self-check: the committed hash must match the committed bytes, so a
        // hand-edit of the file can never slip a mismatched pair through review.
        for (const [name, p] of [
            ["halt", halt],
            ["resume", resume],
        ] as const) {
            const recomputed = blake2AsHex(p.callData, 256)
            if (recomputed !== p.hash) {
                throw new Error(
                    `${name}: hash mismatch, SDK returned ${p.hash} but blake2(callData)=${recomputed}`,
                )
            }
        }

        const reference: ReferenceFile = {
            description:
                "Canonical Snowbridge full halt / full resume governance preimages. " +
                "Reviewed by the Polkadot Fellowship and executed against forked live " +
                "chains by polkadot-ecosystem-tests. Verify an operator-generated " +
                "preimage by comparing its hash against the matching hash here.",
            generatedAt: new Date().toISOString().slice(0, 10),
            assetHubRuntime: runtimeInfo(assetHub),
            bridgeHubRuntime: runtimeInfo(bridgeHub),
            halt: {
                hash: halt.hash,
                callData: halt.callData,
                encodedSize: halt.encodedSize,
            },
            resume: {
                hash: resume.hash,
                callData: resume.callData,
                encodedSize: resume.encodedSize,
            },
        }

        const json = JSON.stringify(reference, null, 2) + "\n"
        process.stdout.write(json)

        // Decode links to stderr so a reviewer can audit the decoded calls (the
        // actual audit), without polluting the JSON on stdout.
        console.error("")
        console.error("Decode (audit) links:")
        console.error(`  halt:   ${halt.decodeUrl}`)
        console.error(`  resume: ${resume.decodeUrl}`)

        if (write) {
            mkdirSync(dirname(OUTPUT_PATH), { recursive: true })
            writeFileSync(OUTPUT_PATH, json)
            console.error("")
            console.error(`Wrote ${OUTPUT_PATH}`)
        }
    } finally {
        await Promise.all([assetHub.disconnect(), bridgeHub.disconnect()])
    }
}

main()
    .then(() => process.exit(0))
    .catch((err) => {
        console.error(err)
        process.exit(1)
    })
