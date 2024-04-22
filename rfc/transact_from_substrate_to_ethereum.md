# RFC: Transact from Substrate to Ethereum


## Summary

This RFC proposes the feature to call transact from Substrate to Ethereum through our bridge, including two PRs separately with https://github.com/Snowfork/snowbridge/pull/1145 for solidity and https://github.com/Snowfork/polkadot-sdk/pull/116 for substrate.

## Explanation

We use penpal for the integration, basically it works as follows:

On penpal end user call the custom extrinsic [transact_to_ethereum](https://github.com/Snowfork/polkadot-sdk/blob/55377cd94b5ef543f1dca2cfd8bcfdd90998dcd4/cumulus/parachains/runtimes/testing/penpal/src/pallets/transact_helper.rs#L70), the parameters of the extrinsic are:

- `target` is the contract address 
- `call` is abi-encoded call data
- `fee` is the execution cost for the call
- `gas_limit` is the max gas limit for the call


It requires penpal to extend the `XcmRouter` to route xcm with destination to Ethereum through our bridge on BH, [config](https://github.com/Snowfork/polkadot-sdk/blob/55377cd94b5ef543f1dca2cfd8bcfdd90998dcd4/cumulus/parachains/runtimes/testing/penpal/src/xcm_config.rs#L376) for the reference.

Worth to note that the [fee config in BridgeTable](https://github.com/Snowfork/polkadot-sdk/blob/55377cd94b5ef543f1dca2cfd8bcfdd90998dcd4/cumulus/parachains/runtimes/testing/penpal/src/xcm_config.rs#L451) should only cover the execution cost on BridgeHub, but not including the cost on Ethereum which is the `fee` as input parameter of the extrinsic.

Then we [charge fees](https://github.com/Snowfork/polkadot-sdk/blob/55377cd94b5ef543f1dca2cfd8bcfdd90998dcd4/cumulus/parachains/runtimes/testing/penpal/src/pallets/transact_helper.rs#L104) from the sender which includes both the execution cost on BridgeHub and cost on Ethereum.

Finally we [deliver](https://github.com/Snowfork/polkadot-sdk/blob/55377cd94b5ef543f1dca2cfd8bcfdd90998dcd4/cumulus/parachains/runtimes/testing/penpal/src/pallets/transact_helper.rs#L107) the xcm to BH.

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
            Transact { 
                origin_kind: SovereignAccount, 
                require_weight_at_most: Weight { ref_time: 0, proof_size: 0 }, 
                call:"0xee9170abfbf9421ad6dd07f6bdec9d89f2b581e02000071468656c6c6f80380100000000002037c77c800200000000000000000000" 
            }, 
            SetTopic([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])]) 
    }
]
```

So the top-level `WithdrawAsset` will withdraw relay token from sovereign account of penpal as fee to pay for the execution cost on BridgeHub.

What we really care about is the internal xcm in `ExportMessage` with [the convert logic in outbound-router](https://github.com/Snowfork/polkadot-sdk/blob/55377cd94b5ef543f1dca2cfd8bcfdd90998dcd4/bridges/snowbridge/primitives/router/src/outbound/mod.rs#L203) it will be converted into a simple `AgentExecuteCommand` which will be relayed and finally executed on Ethereum.

There is a E2E test [transact_from_penpal_to_ethereum](https://github.com/Snowfork/polkadot-sdk/blob/55377cd94b5ef543f1dca2cfd8bcfdd90998dcd4/cumulus/parachains/integration-tests/emulated/tests/bridges/bridge-hub-rococo/src/tests/snowbridge.rs#L566) for demonstration.

On Ethereum side based on the `AgentExecuteCommand` the Agent will [execute the call](https://github.com/Snowfork/snowbridge/blob/0a8dc1e425d495a1bfa217cea6dde520260519ec/contracts/src/AgentExecutor.sol#L48) to finish the whole flow.

## Fee flow

- User represents a user who kicks off an extrinsic on the parachain.
- Parachain represents the source parachain, its sovereign or its agent depending on context.

Sequence|Where|Who|What
-|-|-|-
1|Parachain|User|pays(DOT, Native) to node to execute custom extrinsic; pays (DOT) to Treasury for both delivery cost on BH and execution cost on Ethereum as part of custom extrinsic.
2|Bridge Hub|Parachain|pays(DOT) to Treasury Account for delivery(local fee), check Remote fee passed as expected.
3|Gateway|Relayer|pays(ETH) to validate and execute message.
4|Gateway|Parachain Agent|pays(ETH) to relayer for delivery(reward+refund) and execution.
