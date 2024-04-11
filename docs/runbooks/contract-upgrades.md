---
description: Instructions on how to upgrade Snowbridge contracts
---

# Contract Upgrades

There are two main cases for upgrading Snowbridge contracts.

1. Gateway upgrade: upgrading the Gateway contracts logic.
2. BEEFY upgrade: Upgrading the BEEFY client, which also includes a Gateway upgrade as the BEEFY client set in the Gateway is immutable.

### 1. Building contracts

You need to build the new version of the gateway contract that you wish to deploy. Make sure to set the required variables set in `.envrc` for deployment, see [#build-and-deploy-contracts](initial-deployment-of-gateway-contracts.md#build-and-deploy-contracts "mention")from the initial deployment guide.

```bash
cd web/packages/test
from_start_services=true sh -c "source ./scripts/build-binary.sh && build_contracts" 
```

### 2. Deploying the BEEFY client (Optional, required only for a BEEFY upgrade)

This will deploy a new instance of the BEEFY client. Make sure to set the required variables set in `.envrc` for deployment and that you have generated the initial checkpoint, see [#generate-beefy-state](initial-deployment-of-gateway-contracts.md#generate-beefy-state "mention") from the initial deployment guide. Take note of the BEEFY client address for use in later steps.

TBC: `deploy_beefy_client` does not exist yet.

```bash
cd web/packages/test
from_start_services=true sh -c "source ./scripts/deploy-contracts.sh && deploy_beefy_client"
```

### 3. Deploy the Gateway logic

This will deploy a new instance. You will be required to provide the BeefyClient contract address, either the existing BEEFY client when doing a Gateway upgrade or the newly deployed BEEFY client if doing a BEEFY upgrade. Take note of the Gateway logic address for use in later steps.

```bash
cd web/packages/test
export BEEFY_CLIENT_CONTRACT_ADDRESS=0xBe68fC2d8249eb60bfCf0e71D5A0d2F2e292c4eD
from_start_services=true sh -c "source ./scripts/deploy-contracts.sh && deploy_gateway_logic"
```

### 4. Building the \`snowbridge-preimage\` Tool

To perform an upgrade you need to use the `snowbridge-preimage` tool to build a pre-image of the upgrade. It can be built using the following command. Notice the network is provided and has to match the environment being deployed, the options are `rococo`, `kusama` and `polkadot`. Using `rococo` as an example.

```bash
cd control
cargo build --no-default-features --release --features rococo
```

### 5. Building the Preimage

You can now use the control tool to build the preimage. This will differ based on whether you are the upgrade requires a storage initialization to run.

#### Getting the code hash

`GATEWAY_LOGIC_CONTRACT_ADDRESS` is from step [#id-3.-deploy-the-gateway-logic](contract-upgrades.md#id-3.-deploy-the-gateway-logic "mention").

`INFURA_PROJECT_ID` is the project id for using infura.

Get the code

```
curl -X POST \
    --data '{"jsonrpc":"2.0","method":"eth_getCode","params":["$GATEWAY_LOGIC_CONTRACT_ADDRESS", "latest"],"id":1}' \
    https://sepolia.infura.io/v3/$INFURA_PROJECT_ID \
    -H 'Content-Type: application/json' \
    -H 'Accept: application/json' -s \
    | jq -r .result
```

You need to `keccak256` hash the contract code to generate the `GATEWAY_LOGIC_CODE_HASH`. [Online keccak256 tool](https://emn178.github.io/online-tools/keccak\_256.html).

If there are code changes only an initializer will not be needed. If there are new storage layout members then an initializer will be needed. Do a `diff` between the version that is currently on-chain and the version you are upgrading to and verify.

#### Generating preimage without Initializer

```bash
cd control
./target/release/snowbridge-preimage \
      --logic-address $GATEWAY_LOGIC_CONTRACT_ADDRESS \
      --logic-code-hash $GATEWAY_LOGIC_CODE_HASH
      -- format binary > upgrade.call
```

#### Generating preimage with Initializer

Here we pass the additional parameters required for the initializer (`0x000000000000`) and the gas requirement (`5000`).

```bash
cd control
./target/release/snowbridge-preimage \
      --logic-address $GATEWAY_LOGIC_CONTRACT_ADDRESS \
      --logic-code-hash $GATEWAY_LOGIC_CODE_HASH \
      --initializer \
      --initializer-params 0x000000000000 \
      --initializer-gas 5000 \
      --format binary > upgrade.call
```

### 6. Submitting a referendum with \`opengov-cli\`

Now that we have the preimage we can submit a governance proposal using the `opengov-cli`.

Build the tool from the repo below:

{% embed url="https://github.com/joepetrowski/opengov-cli?tab=readme-ov-file#opengov-cli" %}
opengov-cli tool
{% endembed %}

`upgrade.call` is from [#id-5.-building-the-preimage](contract-upgrades.md#id-5.-building-the-preimage "mention")

```bash
opengov-cli submit-referendum \
    --track $TRACK \
    --network polkadot \
    --network upgrade.call
```

TBC: Confirm track, and test the actual process
