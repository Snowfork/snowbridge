
# Generate Rust bindings for contracts
# Only generate bindings for contracts actually used by the gas estimator
forge bind --module --overwrite \
    --select 'IGatewayV2' \
    --bindings-path src/contracts \
    --root ../contracts

# Substrate chains metadata
# Local westend
subxt metadata --url ws://127.0.0.1:9944 -o runtimes/westend-local/polkadot-metadata.bin
subxt metadata --url ws://127.0.0.1:12144 -o runtimes/asset-hub-westend-local/asset-hub-metadata.bin
subxt metadata --url ws://127.0.0.1:11144 -o runtimes/bridge-hub-westend-local/bridge-hub-metadata.bin
# Westend
subxt metadata --url wss://westend-rpc.polkadot.io -o runtimes/westend/polkadot-metadata.bin
subxt metadata --url wss://westend-asset-hub-rpc.polkadot.io -o runtimes/asset-hub-westend/asset-hub-metadata.bin
subxt metadata --url wss://westend-bridge-hub-rpc.polkadot.io -o runtimes/bridge-hub-westend/bridge-hub-metadata.bin
# Paseo
subxt metadata --url wss://pas-rpc.stakeworld.io -o runtimes/paseo/polkadot-metadata.bin
subxt metadata --url wss://sys.turboflakes.io/asset-hub-paseo -o runtimes/asset-hub-paseo/asset-hub-metadata.bin
subxt metadata --url wss://bridge-hub-paseo.dotters.network -o runtimes/bridge-hub-paseo/bridge-hub-metadata.bin
