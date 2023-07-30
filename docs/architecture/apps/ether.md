# Ether

_Note: "apps" are obsoleted in favour of a design more closely following XCMv3. Docs will follow._&#x20;

Our _Ether_ app allows Etherem users to mint wrapped Ether on our common-good parachain. To do this, they must lock up Ether as collateral.

## Design

Wrapped Ether (wETH) on our parachain is designed to have 128-bits of precision to be interoperable with XCM and other parachains. This contrasts with Ethereum where Ether is represented using 256-bits of precision.

Given that native Ether has 18 decimal places, this means that a maximum of $$3.4*10^{20}$$wrapped ether can be minted, which is still a huge amount.

## Implementation

The app consists of two peer components which communicate via [channels](../channels/):

* [EtherApp smart contract](../../../contracts/contracts/ETHApp.sol)
* [EtherApp pallet](https://github.com/Snowfork/snowbridge/tree/main/parachain/pallets/eth-app)

On the parachain, the wrapped Ether is stored in a FRAME [assets](https://github.com/paritytech/substrate/tree/master/frame/assets) pallet with an asset id of `0`.
