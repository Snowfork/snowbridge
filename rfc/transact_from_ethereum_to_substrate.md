# RFC: Transact from Ethereum to Substrate


## Summary

This RFC proposes the feature to call transact from Ethereum to Substrate through our bridge, including two PRs separately with https://github.com/Snowfork/snowbridge/pull/1141 for solidity and https://github.com/Snowfork/polkadot-sdk/pull/114 for substrate. 

## Explanation

Basically it works as follows:

First on Ethereum to call the transact via [sendCall](https://github.com/Snowfork/snowbridge/blob/bdf4c716c3863ad7c2a83ee870c8c399919c4e26/contracts/src/Gateway.sol#L630), the value of parameter `call` is just the scale-encoded extrinsic on substrate.

Then on BridgeHub with [the convert logic in inbound-router](https://github.com/Snowfork/polkadot-sdk/blob/cd7a64a1ca5b8e1ea6339125c0c966065ada8e70/bridges/snowbridge/primitives/router/src/inbound/mod.rs#L337) the payload will be converted into a xcm which will be sent to the destination chain.

Worth to note that the [BurnAsset](https://github.com/Snowfork/polkadot-sdk/blob/cd7a64a1ca5b8e1ea6339125c0c966065ada8e70/bridges/snowbridge/primitives/router/src/inbound/mod.rs#L353) in the xcm will do nothing on destination chain, included here only for the destination chain to implement a [custom Barrier](https://github.com/Snowfork/polkadot-sdk/blob/cd7a64a1ca5b8e1ea6339125c0c966065ada8e70/cumulus/parachains/runtimes/testing/penpal/src/xcm_config.rs#L227) which inspect the fee as expected(i.e. can cover the transact cost to avoid spamming).


There is a E2E test [transact_from_ethereum_to_penpal_success](https://github.com/Snowfork/polkadot-sdk/blob/cd7a64a1ca5b8e1ea6339125c0c966065ada8e70/cumulus/parachains/integration-tests/emulated/tests/bridges/bridge-hub-rococo/src/tests/snowbridge.rs#L568) 

Check the xcm log on penpal we can see that the transact(System::remark_with_event) is executed as expected.

```
2024-04-17T02:59:27.224538Z TRACE xcm::process_instruction: === UnpaidExecution { weight_limit: Unlimited, check_origin: None }
2024-04-17T02:59:27.224575Z TRACE xcm::process_instruction: === BurnAsset(Assets([Asset { id: AssetId(Location { parents: 1, interior: Here }), fun: Fungible(40000000000) }]))
2024-04-17T02:59:27.224585Z TRACE xcm::process_instruction: === DescendOrigin(X1([PalletInstance(80)]))
2024-04-17T02:59:27.224610Z TRACE xcm::process_instruction: === UniversalOrigin(GlobalConsensus(Ethereum { chain_id: 11155111 }))
2024-04-17T02:59:27.224666Z TRACE xcm::process_instruction: === DescendOrigin(X1([AccountKey20 { network: None, key: [144, 169, 135, 185, 68, 203, 29, 204, 229, 86, 78, 95, 222, 205, 122, 84, 211, 222, 39, 254] }]))
2024-04-17T02:59:27.224678Z TRACE xcm::process_instruction: === Transact { origin_kind: SovereignAccount, require_weight_at_most: Weight { ref_time: 40000000, proof_size: 8000 }, call: "0x00071468656c6c6f" }
2024-04-17T02:59:27.224691Z TRACE xcm::process_instruction::transact: Processing call: RuntimeCall::System(Call::remark_with_event { remark: [104, 101, 108, 108, 111] })
2024-04-17T02:59:27.224699Z TRACE xcm::origin_conversion: SovereignSignedViaLocation origin: Location { parents: 2, interior: X2([GlobalConsensus(Ethereum { chain_id: 11155111 }), AccountKey20 { network: None, key: [144, 169, 135, 185, 68, 203, 29, 204, 229, 86, 78, 95, 222, 205, 122, 84, 211, 222, 39, 254] }]) }, kind: SovereignAccount
2024-04-17T02:59:27.224721Z TRACE xcm::process_instruction::transact: Dispatching with origin: Origin { caller: OriginCaller::system(RawOrigin::Signed(ee99e7e8ac49f08251154c033f827541f4fb8a5b1fc4d6d9b1ab72c103bd3023 (5HTYyQW9...))), filter: "[function ptr]" }
2024-04-17T02:59:27.224828Z TRACE xcm::process_instruction::transact: Dispatch successful: PostDispatchInfo { actual_weight: None, pays_fee: Pays::Yes }
2024-04-17T02:59:27.224838Z TRACE xcm::process_instruction: === SetTopic([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1])
2024-04-17T02:59:27.224847Z TRACE xcm::execute: result: Ok(())
```

## Testing, Security, and Privacy

There is some other E2E tests

- [transact_from_ethereum_to_penpal_insufficient_weight](https://github.com/Snowfork/polkadot-sdk/blob/cd7a64a1ca5b8e1ea6339125c0c966065ada8e70/cumulus/parachains/integration-tests/emulated/tests/bridges/bridge-hub-rococo/src/tests/snowbridge.rs#L624)

- [transact_from_ethereum_to_penpal_insufficient_fee](https://github.com/Snowfork/polkadot-sdk/blob/cd7a64a1ca5b8e1ea6339125c0c966065ada8e70/cumulus/parachains/integration-tests/emulated/tests/bridges/bridge-hub-rococo/src/tests/snowbridge.rs#L665C4-L665C53)

which demonstrates the [custom Barrier](https://github.com/Snowfork/polkadot-sdk/blob/cd7a64a1ca5b8e1ea6339125c0c966065ada8e70/cumulus/parachains/runtimes/testing/penpal/src/xcm_config.rs#L227) on penpal can check the fee to cover the transact cost.

