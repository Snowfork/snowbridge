
# Generate Rust bindings for contracts
forge bind --module --overwrite \
    --select 'IGateway|IUpgradable|WETH9|MockGatewayV2|Token' \
    --bindings-path src/contracts \
    --root ../contracts

# Substrate chains metadata
subxt metadata --url wss://westend-rpc.polkadot.io -o runtimes/westend/polkadot-metadata.bin
subxt metadata --url wss://westend-asset-hub-rpc.polkadot.io -o runtimes/asset-hub-westend/asset-hub-metadata.bin
subxt metadata --url wss://westend-bridge-hub-rpc.polkadot.io -o runtimes/bridge-hub-westend/bridge-hub-metadata.bin
