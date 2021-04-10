//go:generate bash -c "jq .abi ../../ethereum/build/contracts/InboundChannel.json | abigen --abi - --type contract --pkg inbound --out inbound/contract.go"
//go:generate bash -c "jq .abi ../../ethereum/build/contracts/OutboundChannel.json | abigen --abi - --type contract --pkg outbound --out outbound/contract.go"
//go:generate bash -c "jq .abi ../../ethereum/build/contracts/LightClientBridge.json | abigen --abi - --type contract --pkg lightclientbridge --out lightclientbridge/contract.go"
//go:generate bash -c "jq .abi ../../ethereum/build/contracts/ValidatorRegistry.json | abigen --abi - --type contract --pkg validatorregistry --out validatorregistry/contract.go"

package contracts
