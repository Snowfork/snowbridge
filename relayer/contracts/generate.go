//go:generate bash -c "jq .abi ../../ethereum/build/contracts/InboundChannel.json | abigen --abi - --type contract --pkg inbound --out inbound/contract.go"
//go:generate bash -c "jq .abi ../../ethereum/build/contracts/BasicOutboundChannel.json | abigen --abi - --type BasicOutboundChannel --pkg outbound --out outbound/basic.go"
//go:generate bash -c "jq .abi ../../ethereum/build/contracts/IncentivizedOutboundChannel.json | abigen --abi - --type IncentivizedOutboundChannel --pkg outbound --out outbound/incentivized.go"

package contracts
