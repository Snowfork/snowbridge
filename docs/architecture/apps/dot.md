# DOT

_Note: "apps" are obsoleted in favour of a design more closely following XCMv3. Docs will follow._&#x20;

Our _DOT_ app allows Polkadot users to mint wrapped DOT on Ethereum. To do this, they must lock up their native DOT as collateral.

## Design

On Ethereum, wrapped DOT (wDOT) is represented using 256-bits of precision, and has 18 decimal places. This contrasts with Polkadot, where DOT is represented using 128-bits of precision and 10 decimal places.

The choice to extend the decimal places to 18 is to largely fit in with consensus in the Ethereum community that the ERC20 token standard was flawed by its support of arbitrary decimal places. dApps on Ethereum can just assume wDOT has 18 decimal places like most other tokens without having to bother with decimal places.

Obviously this means that if wDOT accounts have very small amounts of dust that cannot be represented using 128-bits with 10 decimal places, then those dust amounts are not redeemable. We don't see this as a problem really.

## Implementation

* [DOTApp smart contract](../../../contracts/contracts/DOTApp.sol)
* [DOTApp pallet](https://github.com/Snowfork/snowbridge/tree/main/parachain/pallets/dot-app)

wDOT is implemented using this ERC-777 contract: [WrappedToken](../../../ethereum/contracts/WrappedToken.sol)
