/**
 * Drift check: verify the committed reference preimage still matches what the
 * SDK builds against the live Asset Hub + Bridge Hub runtimes.
 *
 * Why this exists: the canonical halt / resume preimage is pinned in
 * polkadot-ecosystem-tests (PET) and audited at review time. A BridgeHub or
 * Asset Hub runtime upgrade can change pallet indices, call encoding, or the XCM
 * version and silently move the canonical bytes. This check, run on a schedule
 * in CI, rebuilds both preimages from the SDK and compares them to the reference
 * so the drift surfaces calmly (a CI alert + regenerate-and-PR) rather than as a
 * red "mismatch" in the governance tool during a live incident.
 *
 * The reference is, in order of preference:
 *   1. --pet  : fetched from PET master (the real trust anchor), or PET_REFERENCE_URL
 *   2. default: the local committed copy at reference/halt_reference_preimages.json
 *
 * Exit code 0 = match, 1 = drift (or error). Suitable for a cron CI job.
 *
 * Usage:
 *   npx ts-node src/pet_reference_check.ts          # compare against local committed copy
 *   npx ts-node src/pet_reference_check.ts --pet    # compare against PET master
 *
 * Env overrides: ASSET_HUB_WS, BRIDGE_HUB_WS, PET_REFERENCE_URL.
 */

import { ApiPromise, WsProvider } from "@polkadot/api"
import { governance } from "@snowbridge/api"
import { readFileSync } from "fs"
import { join } from "path"

const ASSET_HUB_WS = process.env.ASSET_HUB_WS ?? "wss://polkadot-asset-hub-rpc.polkadot.io"
const BRIDGE_HUB_WS = process.env.BRIDGE_HUB_WS ?? "wss://polkadot-bridge-hub-rpc.polkadot.io"

const PET_REFERENCE_URL =
    process.env.PET_REFERENCE_URL ??
    "https://raw.githubusercontent.com/open-web3-stack/polkadot-ecosystem-tests/master/packages/shared/src/snowbridge/referencePreimages.json"

const LOCAL_REFERENCE_PATH = join(__dirname, "..", "reference", "halt_reference_preimages.json")

interface PreimageEntry {
    hash: string
    callData: string
    encodedSize: number
}

interface ReferenceFile {
    generatedWith?: string
    generatedAt?: string
    assetHubRuntime?: { specName: string; specVersion: number }
    bridgeHubRuntime?: { specName: string; specVersion: number }
    halt: PreimageEntry
    resume: PreimageEntry
}

async function loadReference(usePet: boolean): Promise<{
    source: string
    reference: ReferenceFile
}> {
    if (usePet) {
        const res = await fetch(PET_REFERENCE_URL)
        if (!res.ok) {
            throw new Error(
                `Failed to fetch PET reference (${res.status}) from ${PET_REFERENCE_URL}`,
            )
        }
        return { source: PET_REFERENCE_URL, reference: (await res.json()) as ReferenceFile }
    }
    const raw = readFileSync(LOCAL_REFERENCE_PATH, "utf-8")
    return { source: LOCAL_REFERENCE_PATH, reference: JSON.parse(raw) as ReferenceFile }
}

function compare(
    label: string,
    built: { hash: string; callData: string },
    ref: PreimageEntry,
): boolean {
    const ok = built.callData === ref.callData && built.hash === ref.hash
    if (ok) {
        console.log(`  ${label}: OK (${ref.hash})`)
    } else {
        console.error(`  ${label}: DRIFT`)
        console.error(`    reference hash: ${ref.hash}`)
        console.error(`    rebuilt   hash: ${built.hash}`)
        if (built.callData !== ref.callData) {
            console.error(`    reference bytes: ${ref.callData}`)
            console.error(`    rebuilt   bytes: ${built.callData}`)
        }
    }
    return ok
}

async function main() {
    const usePet = process.argv.includes("--pet")
    const { source, reference } = await loadReference(usePet)
    console.log(`Reference source: ${source}`)
    if (reference.bridgeHubRuntime) {
        console.log(
            `Reference runtimes: AH ${reference.assetHubRuntime?.specVersion}, ` +
                `BH ${reference.bridgeHubRuntime.specVersion} ` +
                `(generated ${reference.generatedAt ?? "?"})`,
        )
    }

    console.log(`Connecting to AssetHub:  ${ASSET_HUB_WS}`)
    console.log(`Connecting to BridgeHub: ${BRIDGE_HUB_WS}`)
    const [assetHub, bridgeHub] = await Promise.all([
        ApiPromise.create({ provider: new WsProvider(ASSET_HUB_WS) }),
        ApiPromise.create({ provider: new WsProvider(BRIDGE_HUB_WS) }),
    ])

    try {
        console.log(
            `Live runtimes: AH ${assetHub.runtimeVersion.specVersion.toNumber()}, ` +
                `BH ${bridgeHub.runtimeVersion.specVersion.toNumber()}`,
        )

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

        console.log("Comparing rebuilt preimages against reference:")
        const haltOk = compare("halt", halt, reference.halt)
        const resumeOk = compare("resume", resume, reference.resume)

        if (haltOk && resumeOk) {
            console.log("\nMATCH: reference is current.")
            process.exit(0)
        } else {
            console.error(
                "\nDRIFT: a runtime change has moved the canonical preimage. " +
                    "Regenerate with generate_reference_preimages.ts, audit the decoded " +
                    "diff, and open a PET PR to update referencePreimages.json.",
            )
            process.exit(1)
        }
    } finally {
        await Promise.all([assetHub.disconnect(), bridgeHub.disconnect()])
    }
}

main().catch((err) => {
    console.error(err)
    process.exit(1)
})
