# RFC: Deposit fee to beneficiary


## Summary

This RFC proposes a change to make the remaining DOT trapped in pallet-xcm on AssetHub deposited to the beneficiary

## Motivation

- To make the remaining DOT trapped in pallet-xcm on AssetHub deposited to the beneficiary
- Another benefit is when fees left more than ED could be used to create the beneficiary account in case it does not exist.

## Stakeholders

## Explanation

Currently the xcm only [deposit asset to the beneficiary](https://github.com/Snowfork/polkadot-sdk/blob/3f495e56ed01f24a29d341d8928c19cc2fd8f17e/bridges/snowbridge/primitives/router/src/inbound/mod.rs#L292-L293) and
will change to ```DepositAsset { assets: Wild(AllCounted(2)), beneficiary }``` which includes both the asset and fees.

So the xcm instructions forwarded as follows:

```
origin: Location { parents: 1, interior: X1([Parachain(1013)]) }, 
instructions: [ 
    ReceiveTeleportedAsset( Assets([Asset { id: AssetId(Location { parents: 1, interior: Here }), fun: Fungible(10000000000) }]) ), 
    BuyExecution { fees: Asset { id: AssetId(Location { parents: 1, interior: Here }), fun: Fungible(10000000000) }, weight_limit: Unlimited }, 
    DescendOrigin(X1([PalletInstance(80)])),
    UniversalOrigin(GlobalConsensus(Ethereum { chain_id: 11155111 })),
    ReserveAssetDeposited(Assets([Asset { id: AssetId(Location { parents: 2, interior: X2([GlobalConsensus(Ethereum { chain_id: 11155111 }), AccountKey20 { network: None, key: [135, 209, 247, 253, 254, 231, 246, 81, 250, 188, 139, 252, 182, 224, 134, 194, 120, 183, 122, 125] }]) }), fun: Fungible(1000000000000000000) }])), 
    ClearOrigin, 
    DepositAsset { assets: Wild(AllCounted(2)), beneficiary: Location { parents: 0, interior: X1([AccountId32 { network: None, id: [40, 72, 212, 220, 228, 167, 56, 124, 15, 56, 249, 50, 168, 47, 27, 78, 215, 131, 99, 62, 18, 255, 115, 240, 65, 108, 201, 130, 181, 216, 41, 48] }]) } } 
]
```

## Drawbacks

This now places a burden on the destination parachain to store excess DOT in one of their pallets, if it is not configured then the deposit will fail and assets will be trapped. 

It should be fine for AssetHub because relay token(i.e. DOT) is configured as a [fungible asset](https://github.com/Snowfork/polkadot-sdk/blob/3f495e56ed01f24a29d341d8928c19cc2fd8f17e/cumulus/parachains/runtimes/assets/asset-hub-rococo/src/xcm_config.rs#L119) but may not be true for other parachains. Usually no Parachain treats the system asset as the local currency.

So we applies this change only on asset hub and not do this on the destination chain portion.

## Testing, Security, and Privacy

In https://github.com/Snowfork/snowbridge/pull/1174 we change the smoke test `send_token` to send weth to an non-existence account on asset hub, the test still works as expected.

From the xcm log we can see that both the asset and the fee deposited to the beneficiary.

```
2024-04-16T02:50:18.942245Z TRACE xcm::barriers: AllowTopLevelPaidExecutionFrom origin: Location { parents: 1, interior: X1([Parachain(1013)]) }, instructions: [ReceiveTeleportedAsset(Assets([Asset { id: AssetId(Location { parents: 1, interior: Here }), fun: Fungible(10000000000) }])), BuyExecution { fees: Asset { id: AssetId(Location { parents: 1, interior: Here }), fun: Fungible(10000000000) }, weight_limit: Unlimited }, DescendOrigin(X1([PalletInstance(80)])), UniversalOrigin(GlobalConsensus(Ethereum { chain_id: 11155111 })), ReserveAssetDeposited(Assets([Asset { id: AssetId(Location { parents: 2, interior: X2([GlobalConsensus(Ethereum { chain_id: 11155111 }), AccountKey20 { network: None, key: [135, 209, 247, 253, 254, 231, 246, 81, 250, 188, 139, 252, 182, 224, 134, 194, 120, 183, 122, 125] }]) }), fun: Fungible(1000000000000000000) }])), ClearOrigin, DepositAsset { assets: Wild(All), beneficiary: Location { parents: 0, interior: X1([AccountId32 { network: None, id: [40, 72, 212, 220, 228, 167, 56, 124, 15, 56, 249, 50, 168, 47, 27, 78, 215, 131, 99, 62, 18, 255, 115, 240, 65, 108, 201, 130, 181, 216, 41, 48] }]) } }], max_weight: Weight { ref_time: 14541642000, proof_size: 362278 }, properties: Properties { weight_credit: Weight { ref_time: 0, proof_size: 0 }, message_id: Some([200, 234, 242, 47, 44, 176, 123, 172, 70, 121, 223, 10, 102, 14, 113, 21, 237, 135, 252, 253, 78, 50, 172, 38, 159, 101, 64, 38, 91, 187, 210, 111]) }
2024-04-16T02:50:18.942266Z TRACE xcm::process: origin: Some(Location { parents: 1, interior: X1([Parachain(1013)]) }), total_surplus/refunded: Weight { ref_time: 0, proof_size: 0 }/Weight { ref_time: 0, proof_size: 0 }, error_handler_weight: Weight { ref_time: 0, proof_size: 0 }
2024-04-16T02:50:18.942320Z TRACE xcm::process_instruction: === ReceiveTeleportedAsset(Assets([Asset { id: AssetId(Location { parents: 1, interior: Here }), fun: Fungible(10000000000) }]))
2024-04-16T02:50:18.942327Z TRACE xcm::ensure_can_subsume_assets: worst_case_holding_len: 1, holding_limit: 64
2024-04-16T02:50:18.942338Z TRACE xcm::contains: ConcreteAssetFromSystem asset: Asset { id: AssetId(Location { parents: 1, interior: Here }), fun: Fungible(10000000000) }, origin: Location { parents: 1, interior: X1([Parachain(1013)]) }
2024-04-16T02:50:18.942344Z TRACE xcm::fungible_adapter: can_check_in origin: Location { parents: 1, interior: X1([Parachain(1013)]) }, what: Asset { id: AssetId(Location { parents: 1, interior: Here }), fun: Fungible(10000000000) }
2024-04-16T02:50:18.942349Z TRACE xcm::fungible_adapter: check_in origin: Location { parents: 1, interior: X1([Parachain(1013)]) }, what: Asset { id: AssetId(Location { parents: 1, interior: Here }), fun: Fungible(10000000000) }
2024-04-16T02:50:18.942355Z TRACE xcm::fungibles_adapter: check_in origin: Location { parents: 1, interior: X1([Parachain(1013)]) }, what: Asset { id: AssetId(Location { parents: 1, interior: Here }), fun: Fungible(10000000000) }
2024-04-16T02:50:18.942361Z TRACE xcm::fungibles_adapter: check_in origin: Location { parents: 1, interior: X1([Parachain(1013)]) }, what: Asset { id: AssetId(Location { parents: 1, interior: Here }), fun: Fungible(10000000000) }
2024-04-16T02:50:18.942406Z TRACE xcm::fungibles_adapter: check_in origin: Location { parents: 1, interior: X1([Parachain(1013)]) }, what: Asset { id: AssetId(Location { parents: 1, interior: Here }), fun: Fungible(10000000000) }
2024-04-16T02:50:18.942414Z TRACE xcm::nonfungibles_adapter: check_in origin: Location { parents: 1, interior: X1([Parachain(1013)]) }, what: Asset { id: AssetId(Location { parents: 1, interior: Here }), fun: Fungible(10000000000) }, context: XcmContext { origin: Some(Location { parents: 1, interior: X1([Parachain(1013)]) }), message_id: [200, 234, 242, 47, 44, 176, 123, 172, 70, 121, 223, 10, 102, 14, 113, 21, 237, 135, 252, 253, 78, 50, 172, 38, 159, 101, 64, 38, 91, 187, 210, 111], topic: None }
2024-04-16T02:50:18.942431Z TRACE xcm::process_instruction: === BuyExecution { fees: Asset { id: AssetId(Location { parents: 1, interior: Here }), fun: Fungible(10000000000) }, weight_limit: Limited(Weight { ref_time: 14541642000, proof_size: 362278 }) }
2024-04-16T02:50:18.942441Z TRACE xcm::weight: UsingComponents::buy_weight weight: Weight { ref_time: 14541642000, proof_size: 362278 }, payment: AssetsInHolding { fungible: {AssetId(Location { parents: 1, interior: Here }): 10000000000}, non_fungible: {} }, context: XcmContext { origin: Some(Location { parents: 1, interior: X1([Parachain(1013)]) }), message_id: [200, 234, 242, 47, 44, 176, 123, 172, 70, 121, 223, 10, 102, 14, 113, 21, 237, 135, 252, 253, 78, 50, 172, 38, 159, 101, 64, 38, 91, 187, 210, 111], topic: None }
2024-04-16T02:50:18.942454Z TRACE xcm::process_instruction: === DescendOrigin(X1([PalletInstance(80)]))
2024-04-16T02:50:18.942460Z TRACE xcm::process_instruction: === UniversalOrigin(GlobalConsensus(Ethereum { chain_id: 11155111 }))
2024-04-16T02:50:18.942479Z TRACE xcm::process_instruction: === ReserveAssetDeposited(Assets([Asset { id: AssetId(Location { parents: 2, interior: X2([GlobalConsensus(Ethereum { chain_id: 11155111 }), AccountKey20 { network: None, key: [135, 209, 247, 253, 254, 231, 246, 81, 250, 188, 139, 252, 182, 224, 134, 194, 120, 183, 122, 125] }]) }), fun: Fungible(1000000000000000000) }]))
2024-04-16T02:50:18.942487Z TRACE xcm::ensure_can_subsume_assets: worst_case_holding_len: 2, holding_limit: 64
2024-04-16T02:50:18.942499Z TRACE xcm::contains: IsTrustedBridgedReserveLocationForConcreteAsset asset: Asset { id: AssetId(Location { parents: 2, interior: X2([GlobalConsensus(Ethereum { chain_id: 11155111 }), AccountKey20 { network: None, key: [135, 209, 247, 253, 254, 231, 246, 81, 250, 188, 139, 252, 182, 224, 134, 194, 120, 183, 122, 125] }]) }), fun: Fungible(1000000000000000000) }, origin: Location { parents: 2, interior: X1([GlobalConsensus(Ethereum { chain_id: 11155111 })]) }, universal_source: X2([GlobalConsensus(Rococo), Parachain(1000)])
2024-04-16T02:50:18.942508Z TRACE xcm::contains: Case asset: Asset { id: AssetId(Location { parents: 2, interior: X2([GlobalConsensus(Ethereum { chain_id: 11155111 }), AccountKey20 { network: None, key: [135, 209, 247, 253, 254, 231, 246, 81, 250, 188, 139, 252, 182, 224, 134, 194, 120, 183, 122, 125] }]) }), fun: Fungible(1000000000000000000) }, origin: Location { parents: 2, interior: X1([GlobalConsensus(Ethereum { chain_id: 11155111 })]) }
2024-04-16T02:50:18.942549Z TRACE xcm::contains: IsForeignConcreteAsset asset: Asset { id: AssetId(Location { parents: 2, interior: X2([GlobalConsensus(Ethereum { chain_id: 11155111 }), AccountKey20 { network: None, key: [135, 209, 247, 253, 254, 231, 246, 81, 250, 188, 139, 252, 182, 224, 134, 194, 120, 183, 122, 125] }]) }), fun: Fungible(1000000000000000000) }, origin: Location { parents: 2, interior: X1([GlobalConsensus(Ethereum { chain_id: 11155111 })]) }
2024-04-16T02:50:18.942576Z TRACE xcm::process_instruction: === ClearOrigin
2024-04-16T02:50:18.942582Z TRACE xcm::process_instruction: === DepositAsset { assets: Wild(All), beneficiary: Location { parents: 0, interior: X1([AccountId32 { network: None, id: [40, 72, 212, 220, 228, 167, 56, 124, 15, 56, 249, 50, 168, 47, 27, 78, 215, 131, 99, 62, 18, 255, 115, 240, 65, 108, 201, 130, 181, 216, 41, 48] }]) } }
2024-04-16T02:50:18.942595Z TRACE xcm::fungible_adapter: deposit_asset what: Asset { id: AssetId(Location { parents: 1, interior: Here }), fun: Fungible(8792406679) }, who: Location { parents: 0, interior: X1([AccountId32 { network: None, id: [40, 72, 212, 220, 228, 167, 56, 124, 15, 56, 249, 50, 168, 47, 27, 78, 215, 131, 99, 62, 18, 255, 115, 240, 65, 108, 201, 130, 181, 216, 41, 48] }]) }
2024-04-16T02:50:18.942713Z TRACE xcm::fungible_adapter: deposit_asset what: Asset { id: AssetId(Location { parents: 2, interior: X2([GlobalConsensus(Ethereum { chain_id: 11155111 }), AccountKey20 { network: None, key: [135, 209, 247, 253, 254, 231, 246, 81, 250, 188, 139, 252, 182, 224, 134, 194, 120, 183, 122, 125] }]) }), fun: Fungible(1000000000000000000) }, who: Location { parents: 0, interior: X1([AccountId32 { network: None, id: [40, 72, 212, 220, 228, 167, 56, 124, 15, 56, 249, 50, 168, 47, 27, 78, 215, 131, 99, 62, 18, 255, 115, 240, 65, 108, 201, 130, 181, 216, 41, 48] }]) }
2024-04-16T02:50:18.942723Z TRACE xcm::fungibles_adapter: deposit_asset what: Asset { id: AssetId(Location { parents: 2, interior: X2([GlobalConsensus(Ethereum { chain_id: 11155111 }), AccountKey20 { network: None, key: [135, 209, 247, 253, 254, 231, 246, 81, 250, 188, 139, 252, 182, 224, 134, 194, 120, 183, 122, 125] }]) }), fun: Fungible(1000000000000000000) }, who: Location { parents: 0, interior: X1([AccountId32 { network: None, id: [40, 72, 212, 220, 228, 167, 56, 124, 15, 56, 249, 50, 168, 47, 27, 78, 215, 131, 99, 62, 18, 255, 115, 240, 65, 108, 201, 130, 181, 216, 41, 48] }]) }
2024-04-16T02:50:18.942758Z TRACE xcm::fungibles_adapter: deposit_asset what: Asset { id: AssetId(Location { parents: 2, interior: X2([GlobalConsensus(Ethereum { chain_id: 11155111 }), AccountKey20 { network: None, key: [135, 209, 247, 253, 254, 231, 246, 81, 250, 188, 139, 252, 182, 224, 134, 194, 120, 183, 122, 125] }]) }), fun: Fungible(1000000000000000000) }, who: Location { parents: 0, interior: X1([AccountId32 { network: None, id: [40, 72, 212, 220, 228, 167, 56, 124, 15, 56, 249, 50, 168, 47, 27, 78, 215, 131, 99, 62, 18, 255, 115, 240, 65, 108, 201, 130, 181, 216, 41, 48] }]) }
2024-04-16T02:50:18.942951Z TRACE xcm::process_instruction: === SetTopic([200, 234, 242, 47, 44, 176, 123, 172, 70, 121, 223, 10, 102, 14, 113, 21, 237, 135, 252, 253, 78, 50, 172, 38, 159, 101, 64, 38, 91, 187, 210, 111])
2024-04-16T02:50:18.942963Z TRACE xcm::execute: result: Ok(())
2024-04-16T02:50:18.942967Z TRACE xcm::refund_surplus: total_surplus: Weight { ref_time: 0, proof_size: 0 }, total_refunded: Weight { ref_time: 0, proof_size: 0 }, current_surplus: Weight { ref_time: 0, proof_size: 0 }
2024-04-16T02:50:18.942972Z TRACE xcm::refund_surplus: total_refunded: Weight { ref_time: 0, proof_size: 0 }
2024-04-16T02:50:18.943024Z TRACE xcm::process-message: XCM message execution complete, used weight: Weight(ref_time: 14541642000, proof_size: 362278)
```

