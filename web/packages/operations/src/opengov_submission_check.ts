/**
 * Hex-parity check: verify that @snowbridge/api's `buildHaltBridgeSubmissionUrls`
 * produces byte-for-byte the same Asset Hub batch and Collectives extrinsic as
 * joepetrowski/opengov-cli, for a known fixture preimage.
 *
 * The fixture was captured by running opengov-cli locally:
 *
 *   opengov-cli submit-referendum \
 *     --proposal 0x<FIXTURE_PREIMAGE> \
 *     --network polkadot \
 *     --track whitelistedcaller \
 *     --output AppsUiLink
 *
 * Outputs the two known papi.how `data=` payloads recorded in
 * EXPECTED_ASSET_HUB_HEX and EXPECTED_COLLECTIVES_HEX below.
 *
 * Connects to live RPCs by default; override with ASSET_HUB_WS / COLLECTIVES_WS.
 * Note: runtime upgrades on either chain can shift pallet indices or type
 * shapes, in which case the fixture goes stale; capture a fresh one by
 * re-running opengov-cli.
 *
 * Usage: `npx ts-node web/packages/operations/src/opengov_submission_check.ts`
 */

import { ApiPromise, WsProvider } from "@polkadot/api"
import { governance } from "@snowbridge/api"

const ASSET_HUB_WS =
    process.env.ASSET_HUB_WS ?? "wss://polkadot-asset-hub-rpc.polkadot.io"
const COLLECTIVES_WS =
    process.env.COLLECTIVES_WS ?? "wss://polkadot-collectives-rpc.polkadot.io"

// Fixture preimage from a recorded opengov-cli run, the raw bytes you'd pass
// via `--proposal`. This is a halt-bridge preimage (XCM-send to BridgeHub
// + AH frontend halt + max-fee writes).
const FIXTURE_PREIMAGE =
    "0x2804081f0005010100a90f05342f000006020102d96aca89700c5301012000060201826b28be89700c5a0101200006020102ca9a3b000c500101200006020102ca9a3b000c5b0101200006020102ca9a3b000c510001200006020102ca9a3b000c520301200028040c240001000404405fbc5c7ba58845ad1f1a9a7c5bc12fad40ffffffffffffffffffffffffffffffff00040440d0ed50b03e9a49e836dd934b425ba4c340ffffffffffffffffffffffffffffffff"

// The two `data=` payloads opengov-cli emits for the fixture above. Copied
// verbatim from the recorded output.
const EXPECTED_ASSET_HUB_HEX =
    "0x2804080500e10240032804081f0005010100a90f05342f000006020102d96aca89700c5301012000060201826b28be89700c5a0101200006020102ca9a3b000c500101200006020102ca9a3b000c5b0101200006020102ca9a3b000c510001200006020102ca9a3b000c520301200028040c240001000404405fbc5c7ba58845ad1f1a9a7c5bc12fad40ffffffffffffffffffffffffffffffff00040440d0ed50b03e9a49e836dd934b425ba4c340ffffffffffffffffffffffffffffffff3e003f0d023ed76771e81b331f874722bfaef740b3219ddd944fcc33021d53735ede9c0e3ab8000000010a000000"
const EXPECTED_COLLECTIVES_HEX =
    "0x2804043d003e0201cc1f0005010100a10f05082f00000603008840009acbcdc7f00d57411325ef59d71f4861d4c9181e07eca54d62f27a7260951710010a000000"

async function main() {
    console.log(`Connecting to AssetHub: ${ASSET_HUB_WS}`)
    console.log(`Connecting to Collectives: ${COLLECTIVES_WS}`)

    const [assetHub, collectives] = await Promise.all([
        ApiPromise.create({ provider: new WsProvider(ASSET_HUB_WS) }),
        ApiPromise.create({ provider: new WsProvider(COLLECTIVES_WS) }),
    ])

    try {
        const urls = await governance.buildHaltBridgeSubmissionUrls(
            assetHub,
            collectives,
            FIXTURE_PREIMAGE,
        )

        console.log("\n--- SDK output ---")
        console.log(`preimageHash:        ${urls.preimageHash}`)
        console.log(`wrappedPreimageHash: ${urls.wrappedPreimageHash}`)
        console.log(`wrappedPreimageLen:  ${urls.wrappedPreimageLen}`)
        console.log(`\nassetHubBatchUrl:    ${urls.assetHubBatchUrl}`)
        console.log(`fellowshipWhitelist: ${urls.fellowshipWhitelistUrl}`)

        const ahMatch = urls.assetHubBatchCallData === EXPECTED_ASSET_HUB_HEX
        const collectivesMatch =
            urls.fellowshipWhitelistCallData === EXPECTED_COLLECTIVES_HEX

        console.log("\n--- Parity check vs opengov-cli ---")
        reportMatch(
            "Asset Hub batch",
            ahMatch,
            EXPECTED_ASSET_HUB_HEX,
            urls.assetHubBatchCallData,
        )
        reportMatch(
            "Collectives batch",
            collectivesMatch,
            EXPECTED_COLLECTIVES_HEX,
            urls.fellowshipWhitelistCallData,
        )

        if (!ahMatch || !collectivesMatch) {
            process.exitCode = 1
        }
    } finally {
        await Promise.all([assetHub.disconnect(), collectives.disconnect()])
    }
}

function reportMatch(
    label: string,
    match: boolean,
    expected: string,
    actual: string,
) {
    if (match) {
        console.log(`  ${label}: PASS (${actual.length / 2 - 1} bytes)`)
        return
    }
    console.log(`  ${label}: FAIL`)
    console.log(`    expected: ${expected}`)
    console.log(`    actual:   ${actual}`)
    const firstDiff = firstDifference(expected, actual)
    if (firstDiff !== null) {
        console.log(
            `    first diff at char ${firstDiff}: expected '${expected[firstDiff] ?? "<end>"}', got '${actual[firstDiff] ?? "<end>"}'`,
        )
    }
}

function firstDifference(a: string, b: string): number | null {
    const len = Math.min(a.length, b.length)
    for (let i = 0; i < len; i++) {
        if (a[i] !== b[i]) return i
    }
    return a.length === b.length ? null : len
}

main().catch((e) => {
    console.error(e)
    process.exit(1)
})
