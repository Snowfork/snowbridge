# RFC: Introduce Polkadot-native assets to Ethereum


## Summary

This RFC proposes the feature to introduce Polkadot-native assets to Ethereum through our bridge, including two PRs separately with https://github.com/Snowfork/snowbridge/pull/1155 for solidity and https://github.com/Snowfork/polkadot-sdk/pull/128 for substrate. 

## Motivation

Currently only native ERC20 from Ethereum can be bridged to substrate, we do want to introduce Polkadot-native assets to Ethereum.

## Explanation

The basic work flow includes steps as following:

### 1. Register Polkadot-native assets as ERC20

By adding a [dispatchable](https://github.com/Snowfork/polkadot-sdk/blob/2d8f3b13cf61c3ce8e5ea15438c4cfbfe3a26722/bridges/snowbridge/pallets/system/src/lib.rs#L604) to `EthereumControl` pallet to register new Polkadot-native assets called via XCM, this dispatchable will send a message over the bridge to the Agent of the Parachain. 

On Ethereum side the agent will [instantiate a new ERC20 token](https://github.com/Snowfork/snowbridge/blob/07545cf7e8f0321e4ab89d7f5eb52bc85ab3d4c1/contracts/src/Assets.sol#L259) representing the Polkadot-native asset.

There is a E2E test [register_relay_token](https://github.com/Snowfork/polkadot-sdk/blob/5a4f3af6932cfcbae98435cb16f98a2ee8db4812/cumulus/parachains/integration-tests/emulated/tests/bridges/bridge-hub-rococo/src/tests/snowbridge.rs#L578) for demonstration.

### 2. Send Polkadot-native assets via [reserve_transfer_assets](https://github.com/Snowfork/polkadot-sdk/blob/2d8f3b13cf61c3ce8e5ea15438c4cfbfe3a26722/polkadot/xcm/pallet-xcm/src/lib.rs#L1027)

First it requires the source parachain to extend the `XcmRouter` to route xcm with destination to Ethereum through our bridge on BH, [config](https://github.com/Snowfork/polkadot-sdk/blob/2d8f3b13cf61c3ce8e5ea15438c4cfbfe3a26722/cumulus/parachains/runtimes/assets/asset-hub-rococo/src/xcm_config.rs#L681) on AssetHub for the reference.

Also worth to note that the [fee config in BridgeTable](https://github.com/Snowfork/polkadot-sdk/blob/2d8f3b13cf61c3ce8e5ea15438c4cfbfe3a26722/cumulus/parachains/runtimes/assets/asset-hub-rococo/src/xcm_config.rs#L886-L889) should cover the total execution cost on BridgeHub and Ethereum. 

There is a E2E test [send_relay_token_from_substrate_to_ethereum](https://github.com/Snowfork/polkadot-sdk/blob/5a4f3af6932cfcbae98435cb16f98a2ee8db4812/cumulus/parachains/integration-tests/emulated/tests/bridges/bridge-hub-rococo/src/tests/snowbridge.rs#L645) for demonstration.

Check the xcm executed on AssetHub from the log as following it shows that the relay token has been reserved to the sovereign account of Ethereum.

```
2024-04-16T12:22:11.680296Z TRACE xcm::process_instruction: === TransferAsset { assets: Assets([Asset { id: AssetId(Location { parents: 1, interior: Here }), fun: Fungible(100000000000) }]), beneficiary: Location { parents: 2, interior: X1([GlobalConsensus(Ethereum { chain_id: 11155111 })]) } }
2024-04-16T12:22:11.680315Z TRACE xcm::fungible_adapter: internal_transfer_asset what: Asset { id: AssetId(Location { parents: 1, interior: Here }), fun: Fungible(100000000000) }, from: Location { parents: 0, interior: X1([AccountId32 { network: Some(Rococo), id: [142, 175, 4, 21, 22, 135, 115, 99, 38, 201, 254, 161, 126, 37, 252, 82, 135, 97, 54, 147, 201, 18, 144, 156, 178, 38, 170, 71, 148, 242, 106, 72] }]) }, to: Location { parents: 2, interior: X1([GlobalConsensus(Ethereum { chain_id: 11155111 })]) }
2024-04-16T12:22:11.680342Z TRACE xcm::location_conversion: GlobalConsensusParachainConvertsFor universal_source: X2([GlobalConsensus(Rococo), Parachain(1000)]), location: Location { parents: 2, interior: X1([GlobalConsensus(Ethereum { chain_id: 11155111 })]) }
2024-04-16T12:22:11.680549Z TRACE xcm::execute: result: Ok(())
```


The xcm forwarded to BridgeHub as following:
```
instructions: [
    WithdrawAsset(Assets([Asset { id: AssetId(Location { parents: 1, interior: Here }), fun: Fungible(4200000000000) }])), 
    BuyExecution { fees: Asset { id: AssetId(Location { parents: 1, interior: Here }), fun: Fungible(4200000000000) }, weight_limit: Unlimited }, 
    SetAppendix(Xcm([DepositAsset { assets: Wild(AllCounted(1)), beneficiary: Location { parents: 1, interior: X1([Parachain(1000)]) } }])), 
    ExportMessage { 
        network: Ethereum { chain_id: 11155111 }, 
        destination: Here, 
        xcm: Xcm([
            ReserveAssetDeposited(Assets([Asset { id: AssetId(Location { parents: 1, interior: X1([GlobalConsensus(Rococo)]) }), fun: Fungible(100000000000) }])), 
            ClearOrigin, 
            BuyExecution { fees: Asset { id: AssetId(Location { parents: 1, interior: X1([GlobalConsensus(Rococo)]) }), fun: Fungible(100000000000) }, weight_limit: Unlimited }, 
            DepositAsset { assets: Wild(AllCounted(1)), beneficiary: Location { parents: 0, interior: X1([AccountKey20 { network: None, key: [68, 165, 126, 226, 242, 252, 203, 133, 253, 162, 176, 177, 142, 189, 13, 141, 35, 51, 112, 14] }]) } }, 
            SetTopic([156, 140, 10, 35, 194, 14, 239, 123, 235, 56, 179, 99, 185, 189, 107, 206, 228, 222, 106, 10, 227, 75, 47, 41, 171, 186, 195, 157, 172, 237, 251, 50])
        ]) 
    }
]
```

So the top-level `WithdrawAsset` will withdraw relay token from sovereign account of AssetHub as fee to pay for the execution cost on both BridgeHub and Ethereum.

What we really care about is the internal xcm in `ExportMessage` with [the convert logic in outbound-router](https://github.com/Snowfork/polkadot-sdk/blob/5a4f3af6932cfcbae98435cb16f98a2ee8db4812/bridges/snowbridge/primitives/router/src/outbound/mod.rs#L318) it will be converted into a simple `Command` which will be relayed and finally executed on Ethereum.

On Ethereum side based on the `Command` the Agent will [mint foreign token to the recipient](https://github.com/Snowfork/snowbridge/blob/07545cf7e8f0321e4ab89d7f5eb52bc85ab3d4c1/contracts/src/Assets.sol#L269) to finish the whole flow.


### 3. Send Polkadot-native assets back from Ethereum to Substrate via [sendToken](https://github.com/Snowfork/snowbridge/blob/07545cf7e8f0321e4ab89d7f5eb52bc85ab3d4c1/contracts/src/Gateway.sol#L463)

So first on Ethereum the Agent will [burn foreign token](https://github.com/Snowfork/snowbridge/blob/07545cf7e8f0321e4ab89d7f5eb52bc85ab3d4c1/contracts/src/Assets.sol#L235) and the [payload](https://github.com/Snowfork/snowbridge/blob/07545cf7e8f0321e4ab89d7f5eb52bc85ab3d4c1/contracts/src/Assets.sol#L220) will be relayed and finally executed on BridgeHub. 

Then on BridgeHub with [the convert logic in inbound-router](https://github.com/Snowfork/polkadot-sdk/blob/5a4f3af6932cfcbae98435cb16f98a2ee8db4812/bridges/snowbridge/primitives/router/src/inbound/mod.rs#L354) it will be converted into a [xcm](https://github.com/Snowfork/polkadot-sdk/blob/5a4f3af6932cfcbae98435cb16f98a2ee8db4812/bridges/snowbridge/primitives/router/src/inbound/mod.rs#L386-L395) which will be sent to the destination chain.

There is a E2E test [send_relay_token_from_ethereum_to_substrate](https://github.com/Snowfork/polkadot-sdk/blob/5a4f3af6932cfcbae98435cb16f98a2ee8db4812/cumulus/parachains/integration-tests/emulated/tests/bridges/bridge-hub-rococo/src/tests/snowbridge.rs#L725) with the xcm forwarded to AssetHub as following:


```
 instructions: [
    ReceiveTeleportedAsset(Assets([Asset { id: AssetId(Location { parents: 1, interior: Here }), fun: Fungible(4000000000) }])), 
    BuyExecution { fees: Asset { id: AssetId(Location { parents: 1, interior: Here }), fun: Fungible(4000000000) }, weight_limit: Unlimited }, 
    DescendOrigin(X1([PalletInstance(80)])), 
    UniversalOrigin(GlobalConsensus(Ethereum { chain_id: 11155111 })), 
    WithdrawAsset(Assets([Asset { id: AssetId(Location { parents: 1, interior: X1([GlobalConsensus(Rococo)]) }), fun: Fungible(100000000000) }])), 
    ClearOrigin, 
    DepositAsset { assets: Wild(AllCounted(2)), beneficiary: Location { parents: 0, interior: X1([AccountId32 { network: None, id: [142, 175, 4, 21, 22, 135, 115, 99, 38, 201, 254, 161, 126, 37, 252, 82, 135, 97, 54, 147, 201, 18, 144, 156, 178, 38, 170, 71, 148, 242, 106, 72] }]) } }
]
```

Check the xcm executed on AssetHub and from the log as following it shows that the relay token has been withdraw from the sovereign account of Ethereum and then deposit to the beneficiary as expected.

```
2024-04-16T13:22:57.534652Z TRACE xcm::process_instruction: === WithdrawAsset(Assets([Asset { id: AssetId(Location { parents: 1, interior: X1([GlobalConsensus(Rococo)]) }), fun: Fungible(100000000000) }]))
2024-04-16T13:22:57.534659Z TRACE xcm::ensure_can_subsume_assets: worst_case_holding_len: 2, holding_limit: 64
2024-04-16T13:22:57.534668Z TRACE xcm::fungible_adapter: withdraw_asset what: Asset { id: AssetId(Location { parents: 1, interior: X1([GlobalConsensus(Rococo)]) }), fun: Fungible(100000000000) }, who: Location { parents: 2, interior: X1([GlobalConsensus(Ethereum { chain_id: 11155111 })]) }
2024-04-16T13:22:57.534675Z TRACE xcm::fungible_adapter: withdraw_asset what: Asset { id: AssetId(Location { parents: 1, interior: X1([GlobalConsensus(Rococo)]) }), fun: Fungible(100000000000) }, who: Location { parents: 2, interior: X1([GlobalConsensus(Ethereum { chain_id: 11155111 })]) }
2024-04-16T13:22:57.534692Z TRACE xcm::location_conversion: GlobalConsensusParachainConvertsFor universal_source: X2([GlobalConsensus(Rococo), Parachain(1000)]), location: Location { parents: 2, interior: X1([GlobalConsensus(Ethereum { chain_id: 11155111 })]) }
2024-04-16T13:22:57.534823Z TRACE xcm::process_instruction: === ClearOrigin
2024-04-16T13:22:57.534831Z TRACE xcm::process_instruction: === DepositAsset { assets: Wild(AllCounted(2)), beneficiary: Location { parents: 0, interior: X1([AccountId32 { network: None, id: [142, 175, 4, 21, 22, 135, 115, 99, 38, 201, 254, 161, 126, 37, 252, 82, 135, 97, 54, 147, 201, 18, 144, 156, 178, 38, 170, 71, 148, 242, 106, 72] }]) } }
2024-04-16T13:22:57.534858Z TRACE xcm::fungible_adapter: deposit_asset what: Asset { id: AssetId(Location { parents: 1, interior: Here }), fun: Fungible(3959106667) }, who: Location { parents: 0, interior: X1([AccountId32 { network: None, id: [142, 175, 4, 21, 22, 135, 115, 99, 38, 201, 254, 161, 126, 37, 252, 82, 135, 97, 54, 147, 201, 18, 144, 156, 178, 38, 170, 71, 148, 242, 106, 72] }]) }
2024-04-16T13:22:57.534914Z TRACE xcm::fungible_adapter: deposit_asset what: Asset { id: AssetId(Location { parents: 1, interior: X1([GlobalConsensus(Rococo)]) }), fun: Fungible(100000000000) }, who: Location { parents: 0, interior: X1([AccountId32 { network: None, id: [142, 175, 4, 21, 22, 135, 115, 99, 38, 201, 254, 161, 126, 37, 252, 82, 135, 97, 54, 147, 201, 18, 144, 156, 178, 38, 170, 71, 148, 242, 106, 72] }]) }
2024-04-16T13:22:57.534924Z TRACE xcm::fungible_adapter: deposit_asset what: Asset { id: AssetId(Location { parents: 1, interior: X1([GlobalConsensus(Rococo)]) }), fun: Fungible(100000000000) }, who: Location { parents: 0, interior: X1([AccountId32 { network: None, id: [142, 175, 4, 21, 22, 135, 115, 99, 38, 201, 254, 161, 126, 37, 252, 82, 135, 97, 54, 147, 201, 18, 144, 156, 178, 38, 170, 71, 148, 242, 106, 72] }]) }
2024-04-16T13:22:57.534985Z TRACE xcm::process_instruction: === SetTopic([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])
2024-04-16T13:22:57.534995Z TRACE xcm::execute: result: Ok(())
```

## Testing, Security, and Privacy

In https://github.com/Snowfork/snowbridge/pull/1155 we add smoke tests to to cover the 3 basic flows above.

```
cargo test --test register_polkadot_token // register relay token(i.e. ROC|KSM|DOT) on Ethereum
cargo test --test transfer_polkadot_token // transfer relay token from AssetHub to Ethereum
cargo test --test send_polkadot_token // send relay token from Ethereum back to AssetHub
```
