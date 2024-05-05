# RFC: Transact from Substrate to Ethereum


## Summary

This RFC proposes the feature to call transact from Substrate to Ethereum through our bridge, including two PRs separately with 
- https://github.com/Snowfork/snowbridge/pull/1145 for solidity  
- https://github.com/Snowfork/polkadot-sdk/pull/116 for substrate

## Explanation

We use penpal for the integration, basically it works as follows:

On penpal end user call the custom extrinsic [transact_to_ethereum](https://github.com/Snowfork/polkadot-sdk/blob/d8e4424de5a38c9bfcbb4d1920ef5ad873460a35/cumulus/parachains/runtimes/testing/penpal/src/pallets/transact_helper.rs#L92), the parameters of the extrinsic are:

- `target` is the contract address 
- `call` is abi-encoded call data
- `gas_limit` is the max gas limit for the call

It requires penpal to extend the `XcmRouter` to route xcm destination to Ethereum through our bridge, i.e. a [SovereignPaidRemoteExporter](https://github.com/Snowfork/polkadot-sdk/blob/d8e4424de5a38c9bfcbb4d1920ef5ad873460a35/cumulus/parachains/runtimes/testing/penpal/src/xcm_config.rs#L376) for the reference.

Worth to note it's the responsibility of the end user to [create a pre-funded agent](https://github.com/Snowfork/polkadot-sdk/blob/d8e4424de5a38c9bfcbb4d1920ef5ad873460a35/cumulus/parachains/integration-tests/emulated/tests/bridges/bridge-hub-rococo/src/tests/snowbridge.rs#L641) for the execution on Ethereum and the [fee config in BridgeTable](https://github.com/Snowfork/polkadot-sdk/blob/d8e4424de5a38c9bfcbb4d1920ef5ad873460a35/cumulus/parachains/runtimes/testing/penpal/src/xcm_config.rs#L441) should only cover the delivery cost on BridgeHub.

The xcm sent to BH as following:

```
instructions: [
    WithdrawAsset(Assets([Asset { id: AssetId(Location { parents: 1, interior: Here }), fun: Fungible(4000000000) }])), 
    BuyExecution { fees: Asset { id: AssetId(Location { parents: 1, interior: Here }), fun: Fungible(4000000000) }, weight_limit: Unlimited }, 
    SetAppendix(Xcm([DepositAsset { assets: Wild(AllCounted(1)), beneficiary: Location { parents: 1, interior: X1([Parachain(2000)]) } }])), 
    ExportMessage { 
        network: Ethereum { chain_id: 11155111 }, 
        destination: Here, 
        xcm: Xcm([
            DescendOrigin(X1([AccountId32 { network: None, id: [212, 53, 147, 199, 21, 253, 211, 28, 97, 20, 26, 189, 4, 169, 159, 214, 130, 44, 133, 88, 133, 76, 205, 227, 154, 86, 132, 231, 165, 109, 162, 125] }])),
            Transact { 
                origin_kind: SovereignAccount, 
                require_weight_at_most: Weight { ref_time: 0, proof_size: 0 }, 
                call:"0xee9170abfbf9421ad6dd07f6bdec9d89f2b581e02000071468656c6c6f80380100000000002037c77c800200000000000000000000" 
            }, 
            SetTopic([246, 212, 97, 180, 175, 81, 117, 103, 224, 221, 218, 113, 3, 208, 44, 190, 29, 179, 253, 192, 17, 9, 96, 195, 56, 123, 82, 145, 102, 55, 31, 171])
        ])
    }
]
```

The top-level `WithdrawAsset` and `BuyExecution` will withdraw relay token from sovereign account of penpal as fee to pay for the delivery cost on BridgeHub.

What we really care about is the internal xcm in `ExportMessage`, the `Transact` in it is actually the encoded [TransactInfo](https://github.com/Snowfork/polkadot-sdk/blob/d8e4424de5a38c9bfcbb4d1920ef5ad873460a35/bridges/snowbridge/primitives/core/src/outbound.rs#L442) which represents the call on Ethereum and a custom agent derived from the previous `DescendOrigin` which represents the original user who is supposed to execute the transact.

With [the convert logic in outbound-router](https://github.com/Snowfork/polkadot-sdk/blob/d8e4424de5a38c9bfcbb4d1920ef5ad873460a35/bridges/snowbridge/primitives/router/src/outbound/mod.rs#L204) it will be converted into a simple `Command` which will be relayed and finally executed on Ethereum.

Worth to note that the sovereign of Penpal only pays(DOT) for the [delivery(local fee) portion](https://github.com/Snowfork/polkadot-sdk/blob/d8e4424de5a38c9bfcbb4d1920ef5ad873460a35/bridges/snowbridge/primitives/router/src/outbound/mod.rs#L111-L114) to the Treasury on BH. There is no refund for the remote fee portion in this case.

There is a E2E test [transact_from_penpal_to_ethereum](https://github.com/Snowfork/polkadot-sdk/blob/d8e4424de5a38c9bfcbb4d1920ef5ad873460a35/cumulus/parachains/integration-tests/emulated/tests/bridges/bridge-hub-rococo/src/tests/snowbridge.rs#L567) for demonstration.

Finally on Ethereum based on the `Command` the specified agent will [execute the call](https://github.com/Snowfork/snowbridge/blob/606e867b7badc6d356c8f4b56e6b81ee0eb27811/contracts/src/Gateway.sol#L393).

## Fee flow

- User represents a user who kicks off an extrinsic on the parachain.
- Parachain represents the source parachain, its sovereign or its agent depending on context.

Sequence|Where|Who|What
-|-|-|-
1|Parachain|User|pays(DOT, Native) to node to execute custom extrinsic; pays (DOT) to Treasury for both delivery cost on BH and execution cost on Ethereum.
2|Bridge Hub|Parachain|pays(DOT) to Treasury Account for delivery(local portion), only check remote fee passed as expected without charging
3|Gateway|Relayer|pays(ETH) to validate and execute message.
4|Gateway|Parachain Agent|pays(ETH) to relayer for delivery(reward+refund) and execution(except for the gas used to dispatch the transact payload).
5|Gateway|User Agent|pays(ETH) to relayer for the transact dispatch.
