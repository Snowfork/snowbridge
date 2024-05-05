# RFC: Transact from Ethereum to Substrate


## Summary

This RFC proposes the feature to call transact from Ethereum to Substrate through our bridge, including two PRs separately.

- https://github.com/Snowfork/snowbridge/pull/1141 for solidity 
- https://github.com/Snowfork/polkadot-sdk/pull/114 for substrate. 

## Explanation

Basically it works as follows:

First on Ethereum to call the transact via [sendCall](https://github.com/Snowfork/snowbridge/blob/dfa331d9de5b2fa4d76fdd0edc00667a6f21b4bf/contracts/src/Gateway.sol#L630), the value of parameter `call` is just the scale-encoded extrinsic on substrate.

Worth to note that on Ethereum we charge only for the delivery cost on BH, not including the execution cost on destination chain(i.e. `remote_fee` marked as [zero](https://github.com/Snowfork/snowbridge/blob/dfa331d9de5b2fa4d76fdd0edc00667a6f21b4bf/contracts/src/Gateway.sol#L625))

Then on BridgeHub with [the convert logic in inbound-router](https://github.com/Snowfork/polkadot-sdk/blob/9829de7a0e12816c96c820b91abe195af8486269/bridges/snowbridge/primitives/router/src/inbound/mod.rs#L342) the payload will be converted into a [xcm](https://github.com/Snowfork/polkadot-sdk/blob/9829de7a0e12816c96c820b91abe195af8486269/bridges/snowbridge/primitives/router/src/inbound/mod.rs#L356-L368) which will be sent to the destination chain.


There is a E2E test [transact_from_ethereum_to_penpal_success](https://github.com/Snowfork/polkadot-sdk/blob/9829de7a0e12816c96c820b91abe195af8486269/cumulus/parachains/integration-tests/emulated/tests/bridges/bridge-hub-rococo/src/tests/snowbridge.rs#L572)

The xcm forwarded to destination chain is:

```
instructions: [
    DescendOrigin(X1([PalletInstance(80)])), 
    UniversalOrigin(GlobalConsensus(Ethereum { chain_id: 11155111 })), 
    DescendOrigin(X1([AccountKey20 { network: None, key: [238, 145, 112, 171, 251, 249, 66, 26, 214, 221, 7, 246, 189, 236, 157, 137, 242, 181, 129, 224] }])), 
    WithdrawAsset(Assets([Asset { id: AssetId(Location { parents: 0, interior: Here }), fun: Fungible(40000000000) }])), 
    BuyExecution { fees: Asset { id: AssetId(Location { parents: 0, interior: Here }), fun: Fungible(40000000000) }, weight_limit: Unlimited }, 
    Transact { origin_kind: SovereignAccount, require_weight_at_most: Weight { ref_time: 40000000, proof_size: 8000 }, call: "0x00071468656c6c6f" }
]
```

Worth to note that it’s a pre-funded account which represents sovereign of msg.sender to pay fo the execution cost on destination chain.

## Fee Flow

- dApp is represents `msg.sender` or its sovereign depending on context.
- Parachain represents the target parachain, its sovereign or its agent depending on context.

Sequence|Where|Who|What
-|-|-|-
1|Gateway|dApp|pays(ETH, converted to DOT here) Parachain Agent for delivery costs only (no execution cost)
2|Bridge Hub|Relayer|pays(DOT) node for execution
3|Bridge Hub|Parachain Sovereign|pays(DOT) Relayer for delivery (refund+reward)
4|Parachain|dApp|pays(DOT, Native) for execution only.

## Testing, Security, and Privacy

Some other E2E tests for edge cases:

- [transact_from_ethereum_to_penpal_insufficient_weight](https://github.com/Snowfork/polkadot-sdk/blob/9829de7a0e12816c96c820b91abe195af8486269/cumulus/parachains/integration-tests/emulated/tests/bridges/bridge-hub-rococo/src/tests/snowbridge.rs#L637)

- [transact_from_ethereum_to_penpal_insufficient_fee](https://github.com/Snowfork/polkadot-sdk/blob/9829de7a0e12816c96c820b91abe195af8486269/cumulus/parachains/integration-tests/emulated/tests/bridges/bridge-hub-rococo/src/tests/snowbridge.rs#L679)

