# Apps

_Note: "apps" are obsoleted in favour of a design more closely following XCMv3. Docs will follow._&#x20;

Apps are on-chain programs responsible for implementing high-level cross-chain features. For example, bridging ERC20 tokens to Polkadot; or in the opposite direction, bridging DOT to Ethereum.

An app usually consist of two sub-components who act as peers. For example, a smart contract on the Ethereum side, and a FRAME pallet on the Parachain side. The peers communicate using [channels](../channels/).

![Example: Alice sends Ether over the bridge to Bob](../../.gitbook/assets/2)

## Core Apps

The bridge comes with set of core apps: Ether, ERC20, and DOT.&#x20;

### XCM Auto-Forwarding <a href="#_w8fb1tbc1z2" id="_w8fb1tbc1z2"></a>

With our core apps, users can bridge Ether and ERC20 tokens from Ethereum to third-party parachains. Initiated by a single transaction.

Without this feature, users would first have to bridge their tokens to the BridgeHub parachain, and then issue another transaction to transfer the tokens to the final destination parachain.

## Third-party Apps

Currently, the bridge does not support registering third-party apps. Eventually this is a feature we want to support, to allow any parachain to message any smart contract. Our base-layer architecture has been designed to make this possible.

The key task would be to make our [channel](../channels/) APIs permissionless, and to extend our cross-chain messaging protocol to include the necessary information for forwarding messages to third-party parachains.&#x20;

Some of this work is already done. For example, on the Ethereum side, users can authorize third-party dApps to send messages over the bridge on their behalf.
