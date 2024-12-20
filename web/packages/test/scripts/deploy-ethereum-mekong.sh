#!/usr/bin/env bash
set -eux

source scripts/set-env.sh
HOST=$(ifconfig | grep -Eo 'inet (addr:)?([0-9]*\.){3}[0-9]*' | grep -Eo '([0-9]*\.){3}[0-9]*' | grep -v '127.0.0.1')
export output_electra_dir="$output_dir/electra"

start_geth() {
    mkdir -p $output_electra_dir
    mkdir -p $output_electra_dir/ethereum
    cp config/genesis-mekong.json $output_electra_dir
    cp config/jwtsecret $output_electra_dir
    cp config/config.yaml $output_electra_dir
    cp config/genesis.ssz $output_electra_dir

    echo "Test dir: $output_electra_dir"

    echo "Starting geth local node"
    docker run --rm \
      -v "${output_electra_dir}:/mnt" \
      docker.io/ethpandaops/geth:master \
      --datadir /mnt/ethereum \
      --state.scheme=hash \
      init /mnt/genesis-mekong.json
    echo "**********************************"
    docker run --rm -m=12g --memory-reservation=8g --cpus 2 \
      -v "${output_electra_dir}:/mnt" \
      -p 8551:8551 \
      -p 8545:8545 \
      -p 8546:8546 \
      --env 'NODE_OPTIONS=--max-old-space-size=8192' \
      docker.io/ethpandaops/geth:master \
      --networkid 7078815900 \
      --vmdebug \
      --datadir /mnt/ethereum \
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
      --authrpc.jwtsecret mnt/jwtsecret \
      --password /dev/null \
      --rpc.gascap 0 \
      --ws.origins "*" \
      --gcmode archive \
      --syncmode=full \
      --bootnodes "enode://125d2dddd0dc0d34b526910d49592545a1e4fe25139be9d9e0eed396211dbd37aa0c0a7fa0444c315803d814e743f890529b58b9261289d5303e16477c216b39@157.230.225.158:30303?discport=30303,enode://508bff69cbb852337cfbf3db9e58fe66ec3254e6a3960c0ef266a2ab1ea78e12f101bba51e3d2e066947a0b9b315e5b26009af81584510394c1b87a5908dca7b@137.184.72.127:30303?discport=30303,enode://b273c662dd15148162c23a5c8407d3d5dbb35fb331ae0c1c3a80c6bfa2bbe077683b28c0cd59f7a482efc00a64a09273eb4767a173441a79b833fdf705d331ae@152.42.247.97:30303?discport=30303" \
      --state.scheme=hash \
      > "$output_dir/geth.log" 2>&1 &
}

start_lodestar() {
    echo "Starting lodestar local node"

    docker run --rm -m=12g --memory-reservation=8g --cpus 2 \
      -v "${output_electra_dir}:/mnt" \
      -p 9596:9596 \
      --env 'NODE_OPTIONS=--max-old-space-size=8192' \
      docker.io/chainsafe/lodestar:v1.23.1 \
      beacon \
      --eth1.depositContractDeployBlock=0 \
      --rest.address "0.0.0.0" \
      --bootnodes="enr:-Iq4QB2ny1q6gkBjqNRU_e-GTbpcJQcI4i3cIZDea0mnAzGgbUTKH8j81g9PRl_-m40F1V4GFBlqZElrcbGnUj9AjGeGAZL8bgmtgmlkgnY0gmlwhJ3m4Z6Jc2VjcDI1NmsxoQJJ3h8aUO3GJHv-bdvHtsQZ2OEisutelYfGjXO4lSg8BYN1ZHCCIzI,enr:-LK4QF2XD_Fe5H9QMVVwBoDs6P_37eURcFvNTcLzOc60p_XlDKIBleMgudA7nltZ7TyAiOuY0BSQzHsdv5iUs7sFyWQEh2F0dG5ldHOIAwAAAAAAAACEZXRoMpDY3UMGYGN2JAABAAAAAAAAgmlkgnY0gmlwhJ3m4Z6Jc2VjcDI1NmsxoQJJ7y6LF_to7NYQd3BVRW1840gm5r1Lm3lfAfC9Wqmw8YN0Y3CCIyiDdWRwgiMo,enr:-Mm4QPtT8J4rpYkixx-COebnEPreuWv9OpgOGOvM01hqZ19eeySxCxOEEVHl2r2c0BYwBuct_yZhvkLqUQatRORlIP4Bh2F0dG5ldHOIAAAAAAAAAACDY3NjBIRldGgykNjdQwZgY3YkAAEAAAAAAACCaWSCdjSCaXCEibhIf4RxdWljgiMpiXNlY3AyNTZrMaEDjight_62uShKNt4IorH13hfqm7kZzVyFxXKI_qDlsTGIc3luY25ldHMAg3RjcIIjKIN1ZHCCIyg,enr:-Mm4QBL6auezk-Zi385j0PyjkzGwQJW7TdOFZKGZMKTGRkI4fxTSTiHLe7kTvdjhBq4kgjPXvUnFiXR6AisA8a0w2lQBh2F0dG5ldHOIAAAAAAAAAACDY3NjBIRldGgykNjdQwZgY3YkAAEAAAAAAACCaWSCdjSCaXCEmCr3YYRxdWljgiMpiXNlY3AyNTZrMaEDJ4xl2Our0Y7OKsSDX9f908HznXm3PKzmC9zD8OB2d0mIc3luY25ldHMAg3RjcIIjKIN1ZHCCIyg" \
      --eth1.providerUrls "http://$HOST:8545" \
      --execution.urls "http://$HOST:8551" \
      --dataDir "/mnt/lodestar" \
      --paramsFile="/mnt/config.yaml" \
      --genesisStateFile="/mnt/genesis.ssz" \
      --rest.namespace="*" \
      --jwt-secret /mnt/jwtsecret \
      --checkpointSyncUrl https://checkpoint-sync.mekong.ethpandaops.io \
      --chain.archiveStateEpochFrequency 1 \
       > "$output_dir/lodestar.log" 2>&1 &
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
