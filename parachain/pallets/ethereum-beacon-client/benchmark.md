# Motivation
Demonstrate that [FastAggregateVerify](https://datatracker.ietf.org/doc/html/draft-irtf-cfrg-bls-signature-04#section-3.3.4) is the most expensive call in ethereum beacon light client, though in [#13031](https://github.com/paritytech/substrate/pull/13031) Parity team has wrapped some low level host functions for `bls-12-381` but adding a high level host function specific for it is super helpful.

# Method
Test data [here](https://github.com/Snowfork/snowbridge/blob/db4885d517cb495c11d023b95e5621a5ee4ab14e/parachain/pallets/ethereum-beacon-client/src/benchmarking/data_mainnet.rs#L553-L1120) is real from goerli network contains 512 public keys from sync committee.

We choose extrinsic [sync_committee_period_update](https://github.com/Snowfork/snowbridge/blob/1bc2f0b49473324922a9b14d1a2ae6f0173cbbb4/parachain/pallets/ethereum-beacon-client/src/lib.rs#L266) as base line for the benchmark and add 3 auxiliary extrinsics for comparison.

## [update_only_with_verify_signed_header](https://github.com/Snowfork/snowbridge/blob/13b91702e886ea045eebd3285ede22498507d02f/parachain/pallets/ethereum-beacon-client/src/lib.rs#L505)
benchmark update only for [verify_signed_header](https://github.com/Snowfork/snowbridge/blob/13b91702e886ea045eebd3285ede22498507d02f/parachain/pallets/ethereum-beacon-client/src/lib.rs#L1054)

## [update_without_bls_fast_aggregate_verify](https://github.com/Snowfork/snowbridge/blob/13b91702e886ea045eebd3285ede22498507d02f/parachain/pallets/ethereum-beacon-client/src/lib.rs#L522)
benchmark update without [bls_fast_aggregate_verify](https://github.com/Snowfork/snowbridge/blob/13b91702e886ea045eebd3285ede22498507d02f/parachain/pallets/ethereum-beacon-client/src/lib.rs#L1165)

## [update_with_bls_aggregate_but_without_verify](https://github.com/Snowfork/snowbridge/blob/13b91702e886ea045eebd3285ede22498507d02f/parachain/pallets/ethereum-beacon-client/src/lib.rs#L539)
benchmark for [bls_fast_aggregate_without_verify](https://github.com/Snowfork/snowbridge/blob/13b91702e886ea045eebd3285ede22498507d02f/parachain/pallets/ethereum-beacon-client/src/lib.rs#L1205)(i.e. aggregate but ignore verification)

# Result

|extrinsic       | minimum execution time benchmarked(us) |
| --------------------------------------- |--------------------------|
|sync_committee_period_update | 76341                    |
|update_only_with_verify_signed_header | 74687                    |
|update_without_bls_fast_aggregate_verify | 1294|
|update_with_bls_aggregate_but_without_verify | 57445 |

- `verify_signed_header` consumes almost `74687/76341=98%` execution time of the total `sync_committee_period_update`
- `bls_fast_aggregate_verify` consumes almost `(74687-1294)/74687=98%` execution time of the total `verify_signed_header`
- `bls_fast_aggregate_without_verify` consumes almost `57445/74687=77%` execution time of the total `bls_fast_aggregate_verify`