
# Generate Rust bindings for contracts
forge bind --module --overwrite \
    --select 'IGatewayV2' \
    --bindings-path src/contracts \
    --root ../contracts

# Substrate chains metadata
# Local westend
#echo "local"
#subxt metadata --url ws://127.0.0.1:12144 -o runtimes/asset-hub-westend-local/asset-hub-metadata.bin
#subxt metadata --url ws://127.0.0.1:11144 -o runtimes/bridge-hub-westend-local/bridge-hub-metadata.bin
# Westend
echo "westend"
subxt metadata --url wss://westend-asset-hub-rpc.polkadot.io -o runtimes/asset-hub-westend/asset-hub-metadata.bin
subxt metadata --url wss://westend-bridge-hub-rpc.polkadot.io -o runtimes/bridge-hub-westend/bridge-hub-metadata.bin
# Paseo
echo "paseo"
subxt metadata --url wss://sys.turboflakes.io/asset-hub-paseo -o runtimes/asset-hub-paseo/asset-hub-metadata.bin
subxt metadata --url wss://bridge-hub-paseo.dotters.network -o runtimes/bridge-hub-paseo/bridge-hub-metadata.bin
# Polkadot
echo "polkadot"
subxt metadata --url wss://asset-hub-polkadot-rpc.n.dwellir.com -o runtimes/asset-hub-polkadot/asset-hub-metadata.bin
subxt metadata --url wss://bridge-hub-polkadot-rpc.n.dwellir.com -o runtimes/bridge-hub-polkadot/bridge-hub-metadata.bin
