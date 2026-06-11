// ---------------------------------------------------------------------------
// DELIVERABLE FOR polkadot-ecosystem-tests (PET), NOT compiled in this repo.
//
// This file is staged here so the Snowbridge change and its verifying test land
// in one reviewable place. The actual PR copies it (and referencePreimages.json)
// into the PET monorepo:
//
//   packages/polkadot/src/assetHubPolkadot.bridgeHubPolkadot.snowbridgeGovernance.e2e.test.ts
//   packages/shared/src/snowbridge/referencePreimages.json
//
// The two-chain `<chainA>.<chainB>.<suite>.e2e.test.ts` filename matters: PET's
// failure-notification job parses the dot-separated chain segments to route a
// red run to the right per-chain GitHub issue.
//
// Helper names/signatures (setupNetworks, scheduleInlineCallWithOrigin,
// schedulerBlockProvider, check/checkSystemEvents) follow PET master as of the
// research date; confirm them against PET `master` at PR time, the import
// surface there is the source of truth.
// ---------------------------------------------------------------------------

import { assetHubPolkadot, bridgeHubPolkadot } from '@e2e-test/networks/chains'
import { scheduleInlineCallWithOrigin, setupNetworks } from '@e2e-test/shared'
import { blake2AsHex } from '@polkadot/util-crypto'
import { afterAll, beforeEach, describe, expect, test } from 'vitest'

import reference from '@e2e-test/shared/snowbridge/referencePreimages.json'

// u128::MAX, the value the halt preimage writes to both Asset Hub base-fee
// storage items so any outbound send is priced out of existence.
const MAX_U128 = (1n << 128n) - 1n

// Asset Hub base-fee parameter storage keys = twox_128(":NAME:"). These are the
// same `system.setStorage` keys the halt/resume preimage targets; recomputable
// with xxhashAsHex(":BridgeHubEthereumBaseFee:", 128).
// twox_128(":BridgeHubEthereumBaseFee:") and twox_128(":BridgeHubEthereumBaseFeeV2:").
// These also appear verbatim inside reference.halt.callData (the system.setStorage
// keys), recompute with xxhashAsHex(":NAME:", 128, true) to verify.
const FEE_KEYS = {
  v1: '0x5fbc5c7ba58845ad1f1a9a7c5bc12fad',
  v2: '0xd0ed50b03e9a49e836dd934b425ba4c3',
}

// Operating-mode expectations after the halt preimage executes. ethereumSystem
// (the Gateway control pallet) rejects outbound; everything else is fully Halted.
const HALTED_BH = {
  ethereumSystem: 'RejectingOutboundMessages',
  ethereumSystemV2: 'RejectingOutboundMessages',
  ethereumInboundQueue: 'Halted',
  ethereumInboundQueueV2: 'Halted',
  ethereumOutboundQueue: 'Halted',
  ethereumBeaconClient: 'Halted',
} as const

const NORMAL_BH = {
  ethereumSystem: 'Normal',
  ethereumSystemV2: 'Normal',
  ethereumInboundQueue: 'Normal',
  ethereumInboundQueueV2: 'Normal',
  ethereumOutboundQueue: 'Normal',
  ethereumBeaconClient: 'Normal',
} as const

describe('Snowbridge governance: halt + resume preimage', () => {
  let assetHub: Awaited<ReturnType<typeof setupNetworks>>[number]
  let bridgeHub: Awaited<ReturnType<typeof setupNetworks>>[number]

  beforeEach(async () => {
    ;[assetHub, bridgeHub] = await setupNetworks(assetHubPolkadot, bridgeHubPolkadot)
  })

  afterAll(async () => {
    await assetHub?.teardown()
    await bridgeHub?.teardown()
  })

  // Integrity guard: the committed hash must match the committed bytes. This
  // fails instantly (no fork needed) if referencePreimages.json was hand-edited
  // into an inconsistent state, so a bad pair can never reach the live-execution
  // assertions below.
  test('reference hashes match reference bytes', () => {
    expect(blake2AsHex(reference.halt.callData, 256)).toBe(reference.halt.hash)
    expect(blake2AsHex(reference.resume.callData, 256)).toBe(reference.resume.hash)
  })

  // The core test: execute the EXACT committed halt bytes with Root origin (what
  // the whitelist pallet does after a referendum passes), then assert the bridge
  // is actually halted on both chains. Then execute the resume bytes and assert
  // it is fully restored. If a runtime upgrade ever made these bytes fail to
  // execute, PET's 6-hourly cron turns red here, before any real incident.
  test('committed halt preimage halts the bridge; resume restores it', async () => {
    const blockProvider = assetHub.config.properties.schedulerBlockProvider

    // --- HALT -------------------------------------------------------------
    await scheduleInlineCallWithOrigin(
      assetHub,
      reference.halt.callData,
      { system: 'Root' },
      blockProvider,
    )
    // Asset Hub block: dispatches the force_batch (frontend halt + fee writes,
    // and emits the polkadotXcm.send carrying the BridgeHub Transacts).
    await assetHub.dev.newBlock({ count: 1 })

    // Asset Hub effects.
    const frontendMode = await assetHub.api.query.snowbridgeSystemFrontend.operatingMode()
    expect(frontendMode.toString()).toBe('Halted')

    const feeV1 = await assetHub.api.rpc.state.getStorage(FEE_KEYS.v1)
    const feeV2 = await assetHub.api.rpc.state.getStorage(FEE_KEYS.v2)
    expect(assetHub.api.createType('u128', feeV1).toBigInt()).toBe(MAX_U128)
    expect(assetHub.api.createType('u128', feeV2).toBigInt()).toBe(MAX_U128)

    // Bridge Hub block: processes the XCM. messageQueue.Processed == success
    // means every ExpectTransactStatus(Success) passed, i.e. all setOperatingMode
    // Transacts executed (the fallback weight was sufficient).
    await bridgeHub.dev.newBlock({ count: 1 })
    for (const [pallet, mode] of Object.entries(HALTED_BH)) {
      const got = await (bridgeHub.api.query as any)[pallet].operatingMode()
      expect(got.toString(), `BH ${pallet} after halt`).toBe(mode)
    }

    // --- RESUME -----------------------------------------------------------
    await scheduleInlineCallWithOrigin(
      assetHub,
      reference.resume.callData,
      { system: 'Root' },
      blockProvider,
    )
    await assetHub.dev.newBlock({ count: 1 })

    expect(
      (await assetHub.api.query.snowbridgeSystemFrontend.operatingMode()).toString(),
    ).toBe('Normal')
    // Base fees restored to the prod values baked into the resume preimage.
    const rFeeV1 = await assetHub.api.rpc.state.getStorage(FEE_KEYS.v1)
    const rFeeV2 = await assetHub.api.rpc.state.getStorage(FEE_KEYS.v2)
    expect(assetHub.api.createType('u128', rFeeV1).toBigInt()).toBeLessThan(MAX_U128)
    expect(assetHub.api.createType('u128', rFeeV2).toBigInt()).toBeLessThan(MAX_U128)

    await bridgeHub.dev.newBlock({ count: 1 })
    for (const [pallet, mode] of Object.entries(NORMAL_BH)) {
      const got = await (bridgeHub.api.query as any)[pallet].operatingMode()
      expect(got.toString(), `BH ${pallet} after resume`).toBe(mode)
    }
  })
})
