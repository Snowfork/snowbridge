# RFC: Deposit fee to beneficiary


## Summary

This RFC proposes a change to make the remaining DOT trapped in pallet-xcm on AssetHub deposited to the beneficiary

## Motivation

- To make the remaining DOT trapped in pallet-xcm on AssetHub deposited to the beneficiary
- Another benefit is when fees left more than ED could be used to create the beneficiary account in case it does not exist.

## Stakeholders

## Explanation

Currently the xcm only [deposit asset to the beneficiary](https://github.com/Snowfork/polkadot-sdk/blob/3f495e56ed01f24a29d341d8928c19cc2fd8f17e/bridges/snowbridge/primitives/router/src/inbound/mod.rs#L292-L293) and
will change to ```DepositAsset { assets: Wild(All), beneficiary }``` which includes both the asset and fees.

## Drawbacks

This now places a burden on the destination parachain to store excess DOT in one of their pallets, if it is not configured to than the deposit will fail and assets will be trapped. It should be fine for AssetHub because relay token(i.e. DOT) is configured as a fungible asset as https://github.com/Snowfork/polkadot-sdk/blob/3f495e56ed01f24a29d341d8928c19cc2fd8f17e/cumulus/parachains/runtimes/assets/asset-hub-rococo/src/xcm_config.rs#L119 but may not be true for other parachains as usually no Parachain treats the system asset as the local currency.

So we applies this change only on asset hub and not do this on the destination chain portion.

## Testing, Security, and Privacy

In https://github.com/Snowfork/snowbridge/pull/1174 we change the smoke test `send_token` to send weth to an non-existence account on asset hub, the test still works as expected.

