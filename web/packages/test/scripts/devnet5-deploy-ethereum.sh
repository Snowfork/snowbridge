#!/usr/bin/env bash
set -eu

source scripts/set-env.sh
export output_devnet5_dir="/tmp/devnet5"

start_geth() {

    mkdir -p $output_devnet5_dir
    mkdir -p $output_devnet5_dir/ethereum
    mkdir -p $output_devnet5_dir/lodestar
    pushd "$root_dir/.."

    GETH_PATH="go-ethereum-devnet5/build/bin/geth"

    # Install Electra geth binary
    if [ ! -f "$GETH_PATH" ]; then
      echo "Local geth binary not found at $GETH_PATH."
      echo "Cloning and building go-ethereum-devnet5..."

      git clone --single-branch --branch prague-devnet-5 \
        https://github.com/lightclient/go-ethereum.git go-ethereum-devnet5
      pushd go-ethereum-devnet5
      make geth

      ./build/bin/geth version

      popd
    else
      echo "Local geth binary already exists at $GETH_PATH. Skipping clone and build."
    fi

    echo "Starting geth local node"
    ./go-ethereum-devnet5/build/bin/geth \
      --datadir "$output_devnet5_dir/ethereum" \
      --state.scheme=hash \
      init "$config_dir/genesis-devnet5.json"
    ./go-ethereum-devnet5/build/bin/geth \
      --networkid 7088110746 \
      --vmdebug \
      --datadir "$output_devnet5_dir/ethereum" \
      --http \
      --http.api debug,personal,eth,net,web3,txpool,engine \
      --ws --ws.api debug,eth,net,web3 \
      --rpc.allow-unprotected-txs \
      --authrpc.addr 0.0.0.0 \
      --authrpc.vhosts "*" \
      --http \
      --http.api "debug,personal,eth,net,web3,txpool,engine,miner" \
      --http.addr 0.0.0.0 \
      --http.vhosts "*" \
      --http.corsdomain '*' \
      --ws \
      --ws.api "debug,eth,net,web3" \
      --ws.addr 0.0.0.0 \
      --ws.origins "*" \
      --allow-insecure-unlock \
      --authrpc.jwtsecret "$config_dir/jwtsecret" \
      --password /dev/null \
      --rpc.gascap 0 \
      --ws.origins "*" \
      --gcmode archive \
      --syncmode=full \
      --state.scheme=hash \
      --bootnodes "enode://3308a61ede37d5f70218a592fb7f107a8af4172f24929989d65c71638b5ba81483092425db98d4be0ef273ea3703b1a0592a579926dd6d2ea82282bf62e0c700@104.248.20.215:30303?discport=30303,enode://a4959299a00be2caad41cf0f89f31aaf8f4a742c9532eaf8f0da15d8c14838dd0880ab2c8d7018c0ff8cabf8ca95d0ee96888c34873ec80a6681eb78bd6bc9ca@46.101.170.74:30303?discport=30303,enode://1f0c0e56d020b9437cc07e6421d7c025d4cc1b8e8a7386d34687a2cb9a3979fe53b57cbd3c213bc8663b5ba978cf91382c4b03767fb56a1ba77b5f7a373713cc@64.226.82.178:30303?discport=30303" \
      > "$output_dir/geth.log" 2>&1 &

      popd
}

start_lodestar() {
    echo "Starting lodestar local node"
    export LODESTAR_PRESET="mainnet"

    pushd $root_dir/lodestar
    ./lodestar --version
    ./lodestar beacon \
        --eth1.depositContractDeployBlock=0 \
        --enr.ip6 "127.0.0.1" \
        --rest.address "0.0.0.0" \
        --bootnodes="enr:-Iq4QAbOTKfv9ApSdZET7mTp6PQSxgsWeQSkYfc8qrHGjJAKZVIX092F0T97I1snRk8YYmo_5YRupZBEk7zY-nGBY7qGAZRvMMPlgmlkgnY0gmlwhEDiZtmJc2VjcDI1NmsxoQJJ3h8aUO3GJHv-bdvHtsQZ2OEisutelYfGjXO4lSg8BYN1ZHCCIzI,enr:-LK4QF1_PI-uenTqEfm6b_n3nCg0HybOxbjBfY6ApIlVXA9WND_dOJqr2N9LNn-eSzw_li_cB-EviVmHq_O5XSRcGEEIh2F0dG5ldHOIAAAAAIABAACEZXRoMpBSs7OTYHECQAQAAAAAAAAAgmlkgnY0gmlwhEDiZtmJc2VjcDI1NmsxoQLitfN3bNf4js0UkHcj4G4G8Ja7tfFsmuzPzZvTg80Y6oN0Y3CCIyiDdWRwgiMo" \
        --eth1.providerUrls "http://127.0.0.1:8545" \
        --execution.urls "http://127.0.0.1:8551" \
        --dataDir "$output_devnet5_dir/lodestar" \
        --paramsFile="$config_dir/config-devnet5.yaml" \
        --genesisStateFile="$config_dir/genesis-devnet5.ssz" \
        --eth1=true \
        --logLevel debug \
        --rest.namespace="*" \
        --jwt-secret $config_dir/jwtsecret \
        --chain.archiveStateEpochFrequency 1 \
        >"$output_dir/lodestar.log" 2>&1 &
    popd
}

deploy_local() {
    # 1. deploy execution client
    echo "Starting execution node"
    start_geth

    echo "Waiting for geth API to be ready"
    sleep 10

    # 2. deploy consensus client
    echo "Starting beacon node"
    start_lodestar
}

deploy_ethereum() {
    check_tool && rm -rf "$ethereum_data_dir" && deploy_local
}

if [ -z "${from_start_services:-}" ]; then
    echo "start ethereum only!"
    trap kill_all SIGINT SIGTERM EXIT
    deploy_ethereum
    echo "ethereum local nodes started!"
    wait
fi
