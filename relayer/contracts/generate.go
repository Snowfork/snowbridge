//go:generate bash -c "jq .abi ../../ethereum/build/contracts/LightClientBridge.json | abigen --abi - --type contract --pkg lightclientbridge --out lightclientbridge/contract.go"
//go:generate bash -c "jq .abi ../../ethereum/build/contracts/ValidatorRegistry.json | abigen --abi - --type contract --pkg validatorregistry --out validatorregistry/contract.go"
//go:generate bash -c "jq .abi ../../ethereum/build/contracts/BasicInboundChannel.json | abigen --abi - --type BasicInboundChannel --pkg inbound --out inbound/basic.go"
//go:generate bash -c "jq .abi ../../ethereum/build/contracts/IncentivizedInboundChannel.json | abigen --abi - --type IncentivizedInboundChannel --pkg inbound --out inbound/incentivized.go"
//go:generate bash -c "jq .abi ../../ethereum/build/contracts/BasicOutboundChannel.json | abigen --abi - --type BasicOutboundChannel --pkg outbound --out outbound/basic.go"
//go:generate bash -c "jq .abi ../../ethereum/build/contracts/IncentivizedOutboundChannel.json | abigen --abi - --type IncentivizedOutboundChannel --pkg outbound --out outbound/incentivized.go"

package contracts
