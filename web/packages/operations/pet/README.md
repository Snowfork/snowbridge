# PET deliverable: verifiable halt-bridge preimage

These files are staged for a pull request to
[polkadot-ecosystem-tests](https://github.com/open-web3-stack/polkadot-ecosystem-tests)
(PET). They are **not** part of the Snowbridge build (they live outside
`src/**`, use PET's `@e2e-test/*` packages and Vitest, and won't typecheck here).
They are kept in this repo so the SDK change and its verifying test are reviewed
together.

## Why

The governance tool (app.snowbridge.network/governance) generates the halt/resume
preimage, but the tool and the `@snowbridge/api` package can be modified outside
Fellowship control, so an operator cannot blindly trust the bytes it produces.
The Fellowship feedback: the generated preimage still needs to be audited and
proven live during an incident, which is the worst time to be decoding SCALE.

This moves the audit to review time and reduces incident-time verification to a
single string comparison:

1. `referencePreimages.json` is the canonical full-halt / full-resume preimage,
   committed to PET and changed only via a reviewed PR (the audit).
2. The test executes the **exact committed bytes** on Chopsticks forks of live
   Asset Hub + Bridge Hub and asserts the bridge actually halts and resumes.
   PET's `update-known-good` cron re-runs it every ~6 hours against fresh chain
   state, so the reference cannot go stale unnoticed: a runtime upgrade that
   breaks it turns the per-chain notification issue red.
3. During an incident, an operator (or the whitelisting Fellow) compares the
   `hash` shown by the tool against the `hash` in PET `master`. Match = the
   bytes are the reviewed, continuously-proven ones. This check is two strings,
   doable on a phone, and the trust root is PET's GitHub, nothing Snowbridge
   hosts.

## Files and where they go in PET

| Staged here | Destination in PET |
| --- | --- |
| `referencePreimages.json` | `packages/shared/src/snowbridge/referencePreimages.json` |
| `assetHubPolkadot.bridgeHubPolkadot.snowbridgeGovernance.e2e.test.ts` | `packages/polkadot/src/` (same name) |

The two-chain `<chainA>.<chainB>.<suite>.e2e.test.ts` filename is load-bearing:
PET's failure-notification job parses the dot-separated chain segments to route a
red run to the correct per-chain GitHub issue. `bridgeHubPolkadot` and
`assetHubPolkadot` are already configured chains in PET (endpoints, CI ports, and
notification issues all exist), so the PR touches only `packages/shared/src/` and
`packages/polkadot/src/`.

## Before opening the PR

- Confirm the helper import surface against PET `master`: `setupNetworks`,
  `scheduleInlineCallWithOrigin`, `schedulerBlockProvider`, and the
  `check`/`checkSystemEvents` helpers. The draft follows PET conventions as of
  the research date but the repo is the source of truth.
- If a 210-byte scheduler `Inline` call hits a bound, switch to noting the
  preimage and scheduling a `Lookup` (PET's `preimage.ts` + `scheduleLookupCallWithOrigin`).
- Run `yarn test packages/polkadot/src/assetHubPolkadot.bridgeHubPolkadot.snowbridgeGovernance.e2e.test.ts`
  and commit any generated snapshot.
- Ask a Fellowship reviewer to be a required reviewer on the PR, and consider a
  PET `CODEOWNERS` entry for `packages/shared/src/snowbridge/`.

## Regenerating the reference (Snowbridge side)

When an Asset Hub / Bridge Hub runtime upgrade or a halt-scope change moves the
canonical bytes, the drift check in this package (`pet_reference_check.ts`)
alerts. To refresh:

```sh
# from web/packages/operations
pnpm generateReferencePreimages          # writes reference/halt_reference_preimages.json
pnpm checkPetReference                    # confirm local rebuild matches
```

Then copy the regenerated JSON here and into the PET PR, including the
polkadot.js decode links (printed to stderr by the generator) in the PR
description so reviewers audit the decoded calls, not raw hex.
