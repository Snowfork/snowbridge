
# Generate Rust bindings for contracts
forge bind --module --overwrite \
    --select 'IGateway|IUpgradable|WETH9|MockGatewayV2|Token' \
    --bindings-path src/contracts \
    --root ../contracts

# Substrate chains metadata
subxt metadata --url ws://127.0.0.1:9944 -o runtimes/westend/polkadot-metadata.bin
subxt metadata --url ws://127.0.0.1:12144 -o runtimes/asset-hub-westend/asset-hub-metadata.bin
subxt metadata --url ws://127.0.0.1:11144 -o runtimes/bridge-hub-westend/bridge-hub-metadata.bin
subxt metadata --url ws://127.0.0.1:11144 -o runtimes/bridge-hub-westend/bridge-hub-metadata.bin
