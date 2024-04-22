# RFC: Introduce Polkadot-native assets to Ethereum


## Summary

This RFC proposes the feature to introduce Polkadot-native assets to Ethereum through our bridge, including two PRs separately with https://github.com/Snowfork/snowbridge/pull/1155 for solidity and https://github.com/Snowfork/polkadot-sdk/pull/128 for substrate. 


## Explanation

We use native token on Penpal for the integration and the basic work flow includes steps as following:

### 1. Register Polkadot-native assets as ERC20

First by adding a [dispatchable](https://github.com/Snowfork/polkadot-sdk/blob/2d8f3b13cf61c3ce8e5ea15438c4cfbfe3a26722/bridges/snowbridge/pallets/system/src/lib.rs#L604) to `EthereumControl` pallet to register new Polkadot-native assets called via XCM, this dispatchable will send a message over the bridge to the agent of the Parachain. 

On Ethereum the agent will [instantiate a new ERC20 token](https://github.com/Snowfork/snowbridge/blob/07545cf7e8f0321e4ab89d7f5eb52bc85ab3d4c1/contracts/src/Assets.sol#L259) representing the Polkadot-native asset.

There is a E2E test [register_penpal_native_token](https://github.com/Snowfork/polkadot-sdk/blob/5ad44df00259f53bc0dac8ea76f085540fdb23a4/cumulus/parachains/integration-tests/emulated/tests/bridges/bridge-hub-rococo/src/tests/snowbridge.rs#L571) for demonstration.

### 2. Send Polkadot-native assets via [reserve_transfer_assets](https://github.com/Snowfork/polkadot-sdk/blob/2d8f3b13cf61c3ce8e5ea15438c4cfbfe3a26722/polkadot/xcm/pallet-xcm/src/lib.rs#L1027)

First it requires the source parachain to extend the `XcmRouter` to route xcm with destination to Ethereum through our bridge on BH, [config](https://github.com/Snowfork/polkadot-sdk/blob/5ad44df00259f53bc0dac8ea76f085540fdb23a4/cumulus/parachains/runtimes/testing/penpal/src/xcm_config.rs#L401) on Penpal for the reference.

Worth to note that the [fee config in BridgeTable](https://github.com/Snowfork/polkadot-sdk/blob/5ad44df00259f53bc0dac8ea76f085540fdb23a4/cumulus/parachains/runtimes/testing/penpal/src/xcm_config.rs#L465-L468) should cover the total execution cost on BridgeHub and Ethereum in DOT. 

There is a E2E test [send_penpal_native_token_to_ethereum](https://github.com/Snowfork/polkadot-sdk/blob/5ad44df00259f53bc0dac8ea76f085540fdb23a4/cumulus/parachains/integration-tests/emulated/tests/bridges/bridge-hub-rococo/src/tests/snowbridge.rs#L655) for demonstration.

Check the xcm executed on penpal it shows that the native token has been reserved to the sovereign account of Ethereum.

```
2024-04-22T05:08:45.117066Z TRACE xcm::process_instruction: === TransferAsset { assets: Assets([Asset { id: AssetId(Location { parents: 0, interior: Here }), fun: Fungible(100000000000) }]), beneficiary: Location { parents: 2, interior: X1([GlobalConsensus(Ethereum { chain_id: 11155111 })]) } }
2024-04-22T05:08:45.117087Z TRACE xcm::fungible_adapter: internal_transfer_asset what: Asset { id: AssetId(Location { parents: 0, interior: Here }), fun: Fungible(100000000000) }, from: Location { parents: 0, interior: X1([AccountId32 { network: Some(Rococo), id: [212, 53, 147, 199, 21, 253, 211, 28, 97, 20, 26, 189, 4, 169, 159, 214, 130, 44, 133, 88, 133, 76, 205, 227, 154, 86, 132, 231, 165, 109, 162, 125] }]) }, to: Location { parents: 2, interior: X1([GlobalConsensus(Ethereum { chain_id: 11155111 })]) }
2024-04-22T05:08:45.117476Z TRACE xcm::execute: result: Ok(())
```


The xcm forwarded to BridgeHub as following:
```
instructions: [
    WithdrawAsset(Assets([Asset { id: AssetId(Location { parents: 1, interior: Here }), fun: Fungible(4200000000000) }])), 
    BuyExecution { fees: Asset { id: AssetId(Location { parents: 1, interior: Here }), fun: Fungible(4200000000000) }, weight_limit: Unlimited }, 
    SetAppendix(Xcm([DepositAsset { assets: Wild(AllCounted(1)), beneficiary: Location { parents: 1, interior: X1([Parachain(2000)]) } }])), 
    ExportMessage { 
        network: Ethereum { chain_id: 11155111 }, 
        destination: Here, 
        xcm: Xcm([
            ReserveAssetDeposited(Assets([Asset { id: AssetId(Location { parents: 1, interior: X2([GlobalConsensus(Rococo), Parachain(2000)]) }), fun: Fungible(100000000000) }])), 
            ClearOrigin, 
            BuyExecution { fees: Asset { id: AssetId(Location { parents: 1, interior: X2([GlobalConsensus(Rococo), Parachain(2000)]) }), fun: Fungible(100000000000) }, weight_limit: Unlimited }, 
            DepositAsset { assets: Wild(AllCounted(1)), beneficiary: Location { parents: 0, interior: X1([AccountKey20 { network: None, key: [68, 165, 126, 226, 242, 252, 203, 133, 253, 162, 176, 177, 142, 189, 13, 141, 35, 51, 112, 14] }]) } }, 
            SetTopic([156, 140, 10, 35, 194, 14, 239, 123, 235, 56, 179, 99, 185, 189, 107, 206, 228, 222, 106, 10, 227, 75, 47, 41, 171, 186, 195, 157, 172, 237, 251, 50])
        ]) 
    }
]
```

So the top-level `WithdrawAsset` will withdraw relay token from sovereign account of Penpal as fee to pay for the execution cost on both BridgeHub and Ethereum.

What we really care about is the internal xcm in `ExportMessage` with [the convert logic in outbound-router](https://github.com/Snowfork/polkadot-sdk/blob/5a4f3af6932cfcbae98435cb16f98a2ee8db4812/bridges/snowbridge/primitives/router/src/outbound/mod.rs#L318) it will be converted into a simple `Command` which will be relayed and finally executed on Ethereum.

On Ethereum side based on the `Command` the Agent will [mint foreign token to the recipient](https://github.com/Snowfork/snowbridge/blob/07545cf7e8f0321e4ab89d7f5eb52bc85ab3d4c1/contracts/src/Assets.sol#L269) to finish the whole flow.

#### Fee flow

- User represents a user who kicks off an extrinsic on the parachain.
- Parachain represents the source parachain, its sovereign or its agent depending on context.

Sequence|Where|Who|What
-|-|-|-
1|Penpal|User| For `reserve_transfer_assets` pays(DOT, Native) to node to execute custom extrinsic; pays (DOT) to Treasury for both delivery cost on BH and execution cost on Ethereum(i.e. `EthereumBaseFee`).
2|Bridge Hub|Parachain|Pays(DOT) to Treasury Account for delivery(local fee), pays(DOT) to Parachain sovereign for delivery(remote fee), essentially a refund. Remote fee converted to ETH here.
3|Gateway|Relayer|pays(ETH) to validate and execute message.
4|Gateway|Parachain Agent|pays(ETH) to relayer for delivery(reward+refund) and execution.


### 3. Send Polkadot-native assets back from Ethereum to Substrate via [sendToken](https://github.com/Snowfork/snowbridge/blob/07545cf7e8f0321e4ab89d7f5eb52bc85ab3d4c1/contracts/src/Gateway.sol#L463)

So first on Ethereum the Agent will [burn foreign token](https://github.com/Snowfork/snowbridge/blob/07545cf7e8f0321e4ab89d7f5eb52bc85ab3d4c1/contracts/src/Assets.sol#L235) and the [payload](https://github.com/Snowfork/snowbridge/blob/07545cf7e8f0321e4ab89d7f5eb52bc85ab3d4c1/contracts/src/Assets.sol#L220) will be relayed and finally executed on BridgeHub. 

Then on BridgeHub with [the convert logic in inbound-router](https://github.com/Snowfork/polkadot-sdk/blob/5ad44df00259f53bc0dac8ea76f085540fdb23a4/bridges/snowbridge/primitives/router/src/inbound/mod.rs#L354) it will be converted into a [xcm](https://github.com/Snowfork/polkadot-sdk/blob/5ad44df00259f53bc0dac8ea76f085540fdb23a4/bridges/snowbridge/primitives/router/src/inbound/mod.rs#L392-L399) which will be sent to the destination chain.

There is a E2E test [send_penpal_native_token_from_ethereum](https://github.com/Snowfork/polkadot-sdk/blob/5ad44df00259f53bc0dac8ea76f085540fdb23a4/cumulus/parachains/integration-tests/emulated/tests/bridges/bridge-hub-rococo/src/tests/snowbridge.rs#L759) with the xcm forwarded to Penpal as following:


```
 instructions: [
    DescendOrigin(X1([PalletInstance(80)])),
    UniversalOrigin(GlobalConsensus(Ethereum { chain_id: 11155111 })),
    WithdrawAsset(Assets([Asset { id: AssetId(Location { parents: 0, interior: Here }), fun: Fungible(8000000000) }, Asset { id: AssetId(Location { parents: 1, interior: X2([GlobalConsensus(Rococo), Parachain(2000)]) }), fun: Fungible(100000000000) }])),
    BuyExecution { fees: Asset { id: AssetId(Location { parents: 0, interior: Here }), fun: Fungible(8000000000) }, weight_limit: Unlimited },
    ClearOrigin, 
    DepositAsset { assets: Wild(AllCounted(2)), beneficiary: Location { parents: 0, interior: X1([AccountId32 { network: None, id: [142, 175, 4, 21, 22, 135, 115, 99, 38, 201, 254, 161, 126, 37, 252, 82, 135, 97, 54, 147, 201, 18, 144, 156, 178, 38, 170, 71, 148, 242, 106, 72] }]) } }
]
```

Check the xcm executed on Penpal it shows that the native token has been withdraw from the sovereign account of Ethereum and then deposit to the beneficiary as expected.

```
2024-04-22T05:23:54.708694Z TRACE xcm::process: origin: Some(Location { parents: 1, interior: X1([Parachain(1013)]) }), total_surplus/refunded: Weight { ref_time: 0, proof_size: 0 }/Weight { ref_time: 0, proof_size: 0 }, error_handler_weight: Weight { ref_time: 0, proof_size: 0 }
2024-04-22T05:23:54.708707Z TRACE xcm::process_instruction: === DescendOrigin(X1([PalletInstance(80)]))
2024-04-22T05:23:54.708718Z TRACE xcm::process_instruction: === UniversalOrigin(GlobalConsensus(Ethereum { chain_id: 11155111 }))
2024-04-22T05:23:54.708804Z TRACE xcm::process_instruction: === WithdrawAsset(Assets([Asset { id: AssetId(Location { parents: 0, interior: Here }), fun: Fungible(8000000000) }, Asset { id: AssetId(Location { parents: 1, interior: X2([GlobalConsensus(Rococo), Parachain(2000)]) }), fun: Fungible(100000000000) }]))
2024-04-22T05:23:54.708815Z TRACE xcm::ensure_can_subsume_assets: worst_case_holding_len: 2, holding_limit: 64
2024-04-22T05:23:54.708845Z TRACE xcm::fungible_adapter: withdraw_asset what: Asset { id: AssetId(Location { parents: 0, interior: Here }), fun: Fungible(8000000000) }, who: Location { parents: 2, interior: X1([GlobalConsensus(Ethereum { chain_id: 11155111 })]) }
2024-04-22T05:23:54.708956Z TRACE xcm::fungible_adapter: withdraw_asset what: Asset { id: AssetId(Location { parents: 1, interior: X2([GlobalConsensus(Rococo), Parachain(2000)]) }), fun: Fungible(100000000000) }, who: Location { parents: 2, interior: X1([GlobalConsensus(Ethereum { chain_id: 11155111 })]) }
2024-04-22T05:23:54.708966Z TRACE xcm::fungible_adapter: withdraw_asset what: Asset { id: AssetId(Location { parents: 1, interior: X2([GlobalConsensus(Rococo), Parachain(2000)]) }), fun: Fungible(100000000000) }, who: Location { parents: 2, interior: X1([GlobalConsensus(Ethereum { chain_id: 11155111 })]) }
2024-04-22T05:23:54.709091Z TRACE xcm::process_instruction: === BuyExecution { fees: Asset { id: AssetId(Location { parents: 0, interior: Here }), fun: Fungible(8000000000) }, weight_limit: Limited(Weight { ref_time: 7000000000, proof_size: 458752 }) }
2024-04-22T05:23:54.709110Z TRACE xcm::weight: UsingComponents::buy_weight weight: Weight { ref_time: 7000000000, proof_size: 458752 }, payment: AssetsInHolding { fungible: {AssetId(Location { parents: 0, interior: Here }): 8000000000}, non_fungible: {} }, context: XcmContext { origin: Some(Location { parents: 2, interior: X1([GlobalConsensus(Ethereum { chain_id: 11155111 })]) }), message_id: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], topic: None }
2024-04-22T05:23:54.709147Z TRACE xcm::weight: UsingComponents::buy_weight weight: Weight { ref_time: 7000000000, proof_size: 458752 }, payment: AssetsInHolding { fungible: {AssetId(Location { parents: 0, interior: Here }): 8000000000}, non_fungible: {} }, context: XcmContext { origin: Some(Location { parents: 2, interior: X1([GlobalConsensus(Ethereum { chain_id: 11155111 })]) }), message_id: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], topic: None }
2024-04-22T05:23:54.709166Z TRACE xcm::process_instruction: === ClearOrigin
2024-04-22T05:23:54.709174Z TRACE xcm::process_instruction: === DepositAsset { assets: Wild(AllCounted(2)), beneficiary: Location { parents: 0, interior: X1([AccountId32 { network: None, id: [142, 175, 4, 21, 22, 135, 115, 99, 38, 201, 254, 161, 126, 37, 252, 82, 135, 97, 54, 147, 201, 18, 144, 156, 178, 38, 170, 71, 148, 242, 106, 72] }]) } }
2024-04-22T05:23:54.709191Z TRACE xcm::fungible_adapter: deposit_asset what: Asset { id: AssetId(Location { parents: 0, interior: Here }), fun: Fungible(3412480000) }, who: Location { parents: 0, interior: X1([AccountId32 { network: None, id: [142, 175, 4, 21, 22, 135, 115, 99, 38, 201, 254, 161, 126, 37, 252, 82, 135, 97, 54, 147, 201, 18, 144, 156, 178, 38, 170, 71, 148, 242, 106, 72] }]) }
2024-04-22T05:23:54.709350Z TRACE xcm::fungible_adapter: deposit_asset what: Asset { id: AssetId(Location { parents: 1, interior: X2([GlobalConsensus(Rococo), Parachain(2000)]) }), fun: Fungible(100000000000) }, who: Location { parents: 0, interior: X1([AccountId32 { network: None, id: [142, 175, 4, 21, 22, 135, 115, 99, 38, 201, 254, 161, 126, 37, 252, 82, 135, 97, 54, 147, 201, 18, 144, 156, 178, 38, 170, 71, 148, 242, 106, 72] }]) }
2024-04-22T05:23:54.709362Z TRACE xcm::fungible_adapter: deposit_asset what: Asset { id: AssetId(Location { parents: 1, interior: X2([GlobalConsensus(Rococo), Parachain(2000)]) }), fun: Fungible(100000000000) }, who: Location { parents: 0, interior: X1([AccountId32 { network: None, id: [142, 175, 4, 21, 22, 135, 115, 99, 38, 201, 254, 161, 126, 37, 252, 82, 135, 97, 54, 147, 201, 18, 144, 156, 178, 38, 170, 71, 148, 242, 106, 72] }]) }
2024-04-22T05:23:54.709459Z TRACE xcm::process_instruction: === SetTopic([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])
2024-04-22T05:23:54.709470Z TRACE xcm::execute: result: Ok(())
2024-04-22T05:23:54.709477Z TRACE xcm::refund_surplus: total_surplus: Weight { ref_time: 0, proof_size: 0 }, total_refunded: Weight { ref_time: 0, proof_size: 0 }, current_surplus: Weight { ref_time: 0, proof_size: 0 }
2024-04-22T05:23:54.709484Z TRACE xcm::refund_surplus: total_refunded: Weight { ref_time: 0, proof_size: 0 }
2024-04-22T05:23:54.709546Z TRACE xcm::process-message: XCM message execution complete, used weight: Weight(ref_time: 7000000000, proof_size: 458752)
```

#### Fee Flow

- dApp is represents `msg.sender` or its sovereign depending on context.
- Parachain represents the target parachain, its sovereign or its agent depending on context.
- Ethereum Sovereign represents `Location{parent:2,interior:[GlobalConsensus(Ethereum)]}`

Sequence|Where|Who|What
-|-|-|-
1|Gateway|dApp|pays(ETH, converted to DOT here) Parachain Agent for both delivery cost on BH and  execution cost on destination(DOT,Native).
2|Bridge Hub|Relayer|pays(DOT) node for execution
3|Bridge Hub|Parachain Sovereign|pays(DOT) Relayer for delivery (refund+reward)
4|Parachain|Ethereum Sovereign|pays(DOT, Native) for execution only.
