/**
 * Generate the canonical halt / resume preimage reference committed to
 * polkadot-ecosystem-tests. Prints the JSON to stdout (and decode links to
 * stderr); `--write` also writes it locally. Env: ASSET_HUB_WS, BRIDGE_HUB_WS.
 */

import { ApiPromise, WsProvider } from "@polkadot/api"
import { governance } from "@snowbridge/api"
import { blake2AsHex } from "@polkadot/util-crypto"
import { writeFileSync, mkdirSync } from "fs"
import { dirname, join } from "path"

const ASSET_HUB_WS = process.env.ASSET_HUB_WS ?? "wss://polkadot-asset-hub-rpc.polkadot.io"
const BRIDGE_HUB_WS = process.env.BRIDGE_HUB_WS ?? "wss://polkadot-bridge-hub-rpc.polkadot.io"

// Transient (gitignored) output; PET owns the canonical copy.
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

        // Self-check: hash must match bytes.
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

        // Decode links to stderr so the JSON on stdout stays clean.
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
