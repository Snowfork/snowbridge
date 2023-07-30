# ERC20

_Note: "apps" are obsoleted in favour of a design more closely following XCMv3. Docs will follow._&#x20;

Our _ERC20_ app allows Etherem users to mint wrapped ERC20 tokens on our common-good parachain. To do this, they must lock up their tokens as collateral.

## Design

Wrapped tokens on our parachain are designed to have 128-bits of precision to be interoperable with XCM and other parachains. This contrasts with Ethereum where ERC20 tokens are represented using 256-bits.

Given that most native ERC20 tokens have 18 decimal places or less, there is still enough precision to mint $$3.4*10^{20}$$ integral units for most wrapped ERC20 tokens.

To be clear, our app doesn't modify decimal places during conversions. We see decimal places as purely a presentation layer problem.

ERC20 metadata such as _name_, _symbol_ and _decimals_ are also transferred across the bridge, so that they can be required from the parachain side.

### Permissionless token registration

Our design allows any ERC20 token to be bridged across to Polkadot. This of course means that malicious or rebasing tokens could enter the system. Particularly for rebasing tokens, there is no way to adjust the supply of the wrapped token on the Polkadot side.

Our approach is to just document clearly that rebasing tokens are not supported.

## Implementation

The app consists of two peer components which communicate via [channels](../channels/):

* [ERC20App smart contract](../../../contracts/contracts/ERC20App.sol)
* [ERC20App pallet](https://github.com/Snowfork/snowbridge/tree/main/parachain/pallets/erc20-app)

On the parachain, the wrapped tokens are stored in a FRAME [assets](https://github.com/paritytech/substrate/tree/master/frame/assets) pallet using numeric identifiers allocated by our [asset-registry](https://github.com/Snowfork/snowbridge/tree/main/parachain/pallets/asset-registry) pallet.

The ERC20App pallet maintains a mapping of ERC20 token address to pallet-asset asset id. This can be queried by users.
