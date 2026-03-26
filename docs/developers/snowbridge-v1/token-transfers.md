---
description: Sending ERC20 Tokens to Polkadot
---

# Token Transfers

The bridge currently supports sending ERC20 tokens from Ethereum to any Polkadot parachain, and back again.

The bridged tokens are minted in `ForeignAssets` pallet of the AssetHub parachain, and then transferred to the final destination using a [reserve transfer](https://wiki.polkadot.network/docs/learn-xcm-usecases#reserve-asset-transfer).

A token transfer can be initiated with a single transaction to our [Gateway](../../../contracts/src/interfaces/IGateway.sol) contract.

## How to send tokens to a parachain

Sending tokens is usually a single step for the user. However, a preliminary registration step is required for tokens which have not previously been bridged before.

### Token Sending

To send a previously registered token to a destination parachain, send this transaction to the Gateway:

```solidity
/// @dev Send ERC20 tokens to parachain `destinationChain` and deposit into account `destinationAddress`
function sendToken(address token, ParaID destinationChain, MultiAddress destinationAddress, uint128 destinationFee, uint128 amount)
    external
    payable;
```

This function will charge a fee in Ether that can be retrieved ahead of time by calling `quoteSendTokenFee`.
