//go:generate bash -c "jq .abi ../../ethereum/build/contracts/InboundChannel.json | abigen --abi - --type contract --pkg inbound --out inbound/contract.go"
//go:generate bash -c "jq .abi ../../ethereum/build/contracts/OutboundChannel.json | abigen --abi - --type contract --pkg outbound --out outbound/contract.go"

package contracts
