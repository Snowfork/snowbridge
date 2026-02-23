//go:generate bash -c "jq .abi ../contracts/out/BeefyClient.sol/BeefyClient.json | abigen --abi - --type BeefyClient --pkg contracts --out contracts/beefy_client.go"
//go:generate bash -c "jq '[.abi[] | select(.name | IN(\"submitInitial\", \"submitFinal\", \"submitFiatShamir\") | not)]' ../contracts/out/BeefyClientWrapper.sol/BeefyClientWrapper.json | abigen --abi - --type BeefyClientWrapper --pkg contracts --out contracts/beefy_client_wrapper.go"
//go:generate bash -c "jq .abi ../contracts/out/IGateway.sol/IGatewayV2.json | abigen --abi - --type Gateway --pkg contracts --out contracts/gateway.go"
//go:generate bash -c "jq .abi ../contracts/out/IGateway.sol/IGatewayV1.json | abigen --abi - --type Gateway --pkg contractsv1 --out contracts/v1/gateway.go"

package main
