# Infrastructure

This subproject contains the tooling and configuration necessary to provision our testnets

## Deploy parachain

Setup:

```bash
node=../../parachain/target/release/artemis
```

Generate chain spec:

```bash
${node} build-spec --disable-default-bootnode > artemis-rococo.json
```

Now update spec as appropriate, including the correct Ethereum configuration:

```bash
vim artemis-rococo.json
```

Export genesis state and validation code:

```bash
node=../../parachain/target/release/artemis

${node} export-genesis-state --chain artemis-rococo.json --parachain-id 200 > genesis-200.state

${node} export-genesis-wasm > genesis-200.wasm
```

Create chain spec for polkadot:

```bash
/tmp/polkadot/target/release/polkadot build-spec --chain rococo-local --disable-default-bootnode > rococo-local-custom.json

/tmp/polkadot/target/release/polkadot build-spec --chain rococo-local-custom.json --raw --disable-default-bootnode > rococo-local.json
```

```Upload all the artifacts to S3
aws s3 cp artemis-rococo.json s3://snowfork-rococo
aws s3 cp rococo-local.json s3://snowfork-rococo
aws s3 ${node} s3://snowfork-rococo
aws s3 /tmp/polkadot/target/release/polkadot s3://snowfork-rococo
aws s3 .../../relayer/build/artemis-relay s3://snowfork-rococo
```
