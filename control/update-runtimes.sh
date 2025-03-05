
networks=(
  polkadot
  westend
  paseo
)

for network in ${networks[@]}; do
  echo "Updating network $network"
  subxt metadata --url wss://$network-rpc.dwellir.com            -f bytes -o runtimes/$network/polkadot-metadata.bin
  subxt metadata --url wss://asset-hub-$network-rpc.dwellir.com  -f bytes -o runtimes/asset-hub-$network/asset-hub-metadata.bin

  bh_metadata=runtimes/bridge-hub-$network/bridge-hub-metadata.bin
  bh_url=wss://bridge-hub-$network-rpc.dwellir.com 
  if [ "$network" = "paseo" ]; then
    bh_url=wss://bridge-hub-paseo.dotters.network
  fi

  subxt metadata --url $bh_url -f bytes -o $bh_metadata
done

