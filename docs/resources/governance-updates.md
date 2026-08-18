# General Governance Updates

Snowbridge has several governance APIs that can only be executed using a democratic process via Polkadot OpenGov.

These APIs include:

* Updating the Gateway contract on Ethereum
* Updating pricing parameters for fee calculations

These APIs are available on the `EthereumSystem` pallet on BridgeHub. We have also developed a [tool](https://github.com/Snowfork/snowbridge/tree/main/control) for generating calls to these APIs.

## Steps for initiating a governance update

As an example, we will show how to upgrade the Gateway contract on Ethereum.

### 1. Generate the preimage

Deploy the new gateway contract (for an upgrade), then generate the preimage for the governance call. Use **hex** output — `opengov-cli` and the referenda tester both take hex.

```bash
snowbridge-preimage --format hex upgrade PARAMS > preimage.hex
```

The tool prints the preimage hash and size to stderr:

```
Preimage Hash: 0x…   # = blake2_256(preimage); needed to whitelist the call below
Preimage Size: …
```

Most governance calls fan out to more than one chain. For example, `pricing-parameters` sends `set_pricing_parameters` to BridgeHub **and** `set_ethereum_fee` to Asset Hub, so the dry-run below confirms state on both.

### 2. Build the referendum calls

Snowbridge governance goes through the **Whitelisted Caller** track, which requires two referenda:

* A Fellowship referendum on Collectives that whitelists the call.
* A public referendum on Asset Hub that dispatches the whitelisted call.

Use [opengov-cli](https://github.com/joepetrowski/opengov-cli) to generate the calls. The large `--output-len-limit` forces it to print full call data instead of a hash:

```bash
opengov-cli submit-referendum \
    --proposal preimage.hex \
    --network polkadot \
    --track whitelistedcaller \
    --output CallData \
    --output-len-limit 100000000
```

From the output, note three pieces of call data:

* the `preimage.note_preimage` call for the public referendum,
* the `referenda.submit` call for the public referendum,
* the Asset Hub `Whitelist.whitelist_call(hash)` (embedded in the Fellowship output) — its hash is the `Preimage Hash` from step 1.

### 3. Dry-run against forked Asset Hub + Bridge Hub

Use [polkadot-referenda-tester](https://github.com/karolk91/polkadot-referenda-tester) to fork the live chains, create the referendum, force-approve it, and execute it. We fork **Asset Hub** (where the referendum runs) and add **Bridge Hub** as an additional chain so the XCM it emits is delivered and its state changes are observable.

Because the Whitelisted Caller track checks that the Fellowship whitelisted the call, we whitelist it on the fork with `--pre-call`/`--pre-origin Root` (injecting `Whitelist.whitelist_call(hash)` as Root) instead of running the companion Fellowship referendum:

```bash
npx -y github:karolk91/polkadot-referenda-tester test \
    --governance-chain-url wss://polkadot-asset-hub-rpc.polkadot.io,<AH_BLOCK> \
    --additional-chains    wss://polkadot-bridge-hub-rpc.polkadot.io,<BH_BLOCK> \
    --call-to-note-preimage-for-governance-referendum 0x<note_preimage_call> \
    --call-to-create-governance-referendum            0x<submit_call> \
    --pre-call  0x<whitelist_call> \
    --pre-origin Root \
    --no-cleanup
```

Notes:

* Pin the fork blocks (`url,<BLOCK>`) so the run is reproducible and the local `.chopsticks-db` cache is reused across runs.
* `--no-cleanup` leaves both forks running. Connect Polkadot-JS to each fork to confirm state — for example the Asset Hub fee storage and `EthereumSystem.PricingParameters` on Bridge Hub.

A worked end-to-end example that runs all three steps (control tool → opengov-cli → tester, including extracting the whitelist call) lives at [`control/test-pricing-ref.sh`](https://github.com/Snowfork/snowbridge/blob/main/control/test-pricing-ref.sh).

A more [complicated](test-runtime-upgrades.md) testing scenario involves upgrading BridgeHub with new code first, and then calling a governance API.

### 4. Submit via OpenGov

Once the dry-run passes, generate the real submission links and submit on the Whitelisted Caller track:

```bash
opengov-cli submit-referendum --proposal preimage.hex --network polkadot --track whitelistedcaller --after 100 --output-len-limit 100 --output AppsUiLink
```
