# General Governance Updates

Snowbridge has several governance APIs that can only be executed using a democratic process via Polkadot OpenGov.

These APIs include:

* Updating the Gateway contract on Ethereum
* Updating pricing parameters for fee calculations

These APIs are available on the `EthereumSystem` pallet on BridgeHub. We have also developed a [tool](https://github.com/Snowfork/snowbridge/tree/main/control) for generating calls to these APIs.

## Steps for initiating a governance update

As an example, we will show how to upgrade the Gateway contract on Ethereum.

### **Generate the preimage**

Deploy the new gateway contract, and then generate a preimage for calling `EthereumSystem.upgrade`

```bash
snowbridge-preimage --format binary upgrade PARAMS > preimage.bin
 
```

### Test the update in chopsticks

The `snowbridge-preimage` tool will also generate a helper script `chopsticks-execute-upgrade.js` to execute the update in simulated chopsticks environment.

1. Run chopsticks and fork Polkadot, AssetHub, and BridgeHub, using these [configs](https://github.com/Snowfork/snowbridge/tree/main/control/chopsticks)

```
chopsticks xcm -r polkadot.yml -p polkadot-asset-hub.yml -p polkadot-bridge-hub.yml
```

2. Once the chopsticks environment has been initialized, connect to BridgeHub in Polkadot-JS, and execute the contents of `chopsticks-execute-upgrade.js` in the Polkadot-JS Javascript console.

A more [complicated](test-runtime-upgrades.md) testing scenario would involve having to upgrade BridgeHub with new code, and then calling a governance API.

### OpenGov

The next step involves submitting the proposal to the Whitelisted Caller track in OpenGov.

This actually involves two referendums:

* A referendum on the Collectives chain where the technical fellowship vote to whitelist the preimage.
* A public referendum on Polkadot where the general public vote to execute the whitelisted preimage.

We use the tool [opengov-cli](https://github.com/joepetrowski/opengov-cli) to generate the various calls required to setup these referendums.

```
opengov-cli submit-referendum --proposal preimage.hex --network polkadot --track whitelisted-caller --after 100 --output-len-limit 100 --output AppsUiLink
```
