//go:generate bash -c "jq .abi ../contracts/out/BeefyClient.sol/BeefyClient.json | abigen --abi - --type BeefyClient --pkg contracts --out contracts/beefy_client.go"
//go:generate bash -c "jq .abi ../contracts/out/IGateway.sol/IGatewayV2.json | abigen --abi - --type Gateway --pkg contracts --out contracts/gateway.go"
//go:generate bash -c "jq .abi ../contracts/out/IGateway.sol/IGatewayV1.json | abigen --abi - --type Gateway --pkg contractsv1 --out contracts/v1/gateway.go"
//go:generate bash -c "jq .abi ../contracts/out/MultiCall3.sol/MultiCall3.json | abigen --abi - --type MultiCall3 --pkg contracts --out contracts/multicall3.go"

package main
