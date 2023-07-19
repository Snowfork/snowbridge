//go:generate bash -c "jq .abi ../../core/packages/contracts/out/OpaqueProof.sol/OpaqueProof.json | abigen --abi - --type OpaqueProof --pkg contracts --out opaque_proof.go"
//go:generate bash -c "jq .abi ../../core/packages/contracts/out/BeefyClient.sol/BeefyClient.json | abigen --abi - --type BeefyClient --pkg contracts --out beefy_client.go"
//go:generate bash -c "jq .abi ../../core/packages/contracts/out/IGateway.sol/IGateway.json | abigen --abi - --type Gateway --pkg contracts --out gateway.go"

package contracts
