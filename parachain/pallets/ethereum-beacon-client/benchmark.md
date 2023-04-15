# Motivation
Demonstrate that [FastAggregateVerify](https://datatracker.ietf.org/doc/html/draft-irtf-cfrg-bls-signature-04#section-3.3.4) is the most expensive call in ethereum beacon light client, though in [#13031](https://github.com/paritytech/substrate/pull/13031) Parity team has wrapped some low level host functions for `bls-12-381` but adding a high level host function specific for it is super helpful.

# Method
We add several benchmarks as following to demonstrate [bls_fast_aggregate_verify](https://github.com/Snowfork/snowbridge/blob/826ffc3f279899bbaa8bf5c805f250201c5b4365/parachain/pallets/ethereum-beacon-client/src/lib.rs#L823) is the main bottleneck. Test data [here](https://github.com/Snowfork/snowbridge/blob/826ffc3f279899bbaa8bf5c805f250201c5b4365/parachain/pallets/ethereum-beacon-client/src/benchmarking/data_mainnet.rs#L553-L1120) is real from goerli network which contains 512 public keys from sync committee.


## [sync_committee_period_update](https://github.com/Snowfork/snowbridge/blob/826ffc3f279899bbaa8bf5c805f250201c5b4365/parachain/pallets/ethereum-beacon-client/src/benchmarking/mod.rs#L69)
Base line benchmark for extrinsic [sync_committee_period_update](https://github.com/Snowfork/snowbridge/blob/826ffc3f279899bbaa8bf5c805f250201c5b4365/parachain/pallets/ethereum-beacon-client/src/lib.rs#L295)


## [bls_fast_aggregate_verify](https://github.com/Snowfork/snowbridge/blob/826ffc3f279899bbaa8bf5c805f250201c5b4365/parachain/pallets/ethereum-beacon-client/src/benchmarking/mod.rs#L193)
Subfunction of [verify_signed_header](#verify_signed_header) with [bls_fast_aggregate_verify](https://github.com/Snowfork/snowbridge/blob/826ffc3f279899bbaa8bf5c805f250201c5b4365/parachain/pallets/ethereum-beacon-client/src/lib.rs#L823) only

## [bls_aggregate_pubkey](https://github.com/Snowfork/snowbridge/blob/826ffc3f279899bbaa8bf5c805f250201c5b4365/parachain/pallets/ethereum-beacon-client/src/benchmarking/mod.rs#L176)
Subfunction of [bls_fast_aggregate_verify](#bls_fast_aggregate_verify) with [bls_aggregate_pubkey](https://github.com/Snowfork/snowbridge/blob/826ffc3f279899bbaa8bf5c805f250201c5b4365/parachain/pallets/ethereum-beacon-client/src/lib.rs#L843) only

## [bls_verify_message](https://github.com/Snowfork/snowbridge/blob/826ffc3f279899bbaa8bf5c805f250201c5b4365/parachain/pallets/ethereum-beacon-client/src/benchmarking/mod.rs#L183)
Subfunction of [bls_fast_aggregate_verify](#bls_fast_aggregate_verify) with [bls_verify_message](https://github.com/Snowfork/snowbridge/blob/826ffc3f279899bbaa8bf5c805f250201c5b4365/parachain/pallets/ethereum-beacon-client/src/lib.rs#L856) only


# Result

## hardware spec
Run benchmark in a EC2 instance 
```
cargo run --release --bin polkadot-parachain --features runtime-benchmarks -- benchmark machine --base-path /mnt/scratch/benchmark

+----------+----------------+-------------+-------------+-------------------+
| Category | Function       | Score       | Minimum     | Result            |
+===========================================================================+
| CPU      | BLAKE2-256     | 1.08 GiBs   | 1.00 GiBs   | ✅ Pass (107.5 %) |
|----------+----------------+-------------+-------------+-------------------|
| CPU      | SR25519-Verify | 568.87 KiBs | 666.00 KiBs | ❌ Fail ( 85.4 %) |
|----------+----------------+-------------+-------------+-------------------|
| Memory   | Copy           | 13.67 GiBs  | 14.32 GiBs  | ✅ Pass ( 95.4 %) |
|----------+----------------+-------------+-------------+-------------------|
| Disk     | Seq Write      | 334.35 MiBs | 450.00 MiBs | ❌ Fail ( 74.3 %) |
|----------+----------------+-------------+-------------+-------------------|
| Disk     | Rnd Write      | 143.59 MiBs | 200.00 MiBs | ❌ Fail ( 71.8 %) |
+----------+----------------+-------------+-------------+-------------------+
```

## benchmark

```
cargo run --release --bin polkadot-parachain \
--features runtime-benchmarks \
-- \
benchmark pallet \
--base-path /mnt/scratch/benchmark \
--chain=bridge-hub-rococo-dev \
--pallet=snowbridge_ethereum_beacon_client \
--extrinsic="*" \
--execution=wasm --wasm-execution=compiled \
--steps 50 --repeat 20 \
--output ./parachains/runtimes/bridge-hubs/bridge-hub-rococo/src/weights/snowbridge_ethereum_beacon_client.rs
```

### [Weights](https://github.com/Snowfork/cumulus/blob/ron/benchmark-beacon-bridge/parachains/runtimes/bridge-hubs/bridge-hub-rococo/src/weights/snowbridge_ethereum_beacon_client.rs)

|extrinsic       | minimum execution time benchmarked(us) |
| --------------------------------------- |----------------------------------------|
|sync_committee_period_update | 125_431                                |                              
|bls_fast_aggregate_verify| 123_207                                |
|bls_aggregate_pubkey | 94_487                                  |
|bls_verify_message | 28_368                                  |

- [bls_fast_aggregate_verify](#bls_fast_aggregate_verify) consumes almost 96% execution time of [verify_signed_header](#verify_signed_header)

- [bls_aggregate_pubkey](#bls_aggregate_pubkey) consumes almost 76% execution time of [bls_fast_aggregate_verify](#bls_fast_aggregate_verify)

- [bls_verify_message](#bls_verify_message) consumes almost the left 23% execution time of [bls_fast_aggregate_verify](#bls_fast_aggregate_verify)

# Conclusion

A high level host function specific for  [bls_fast_aggregate_verify](https://github.com/Snowfork/snowbridge/blob/826ffc3f279899bbaa8bf5c805f250201c5b4365/parachain/pallets/ethereum-beacon-client/src/lib.rs#L823) is super helpful.
