//go:generate bash -c "jq .abi ../../core/packages/contracts/out/OpaqueProof.sol/OpaqueProof.json | abigen --abi - --type OpaqueProof --pkg contracts --out opaque_proof.go"
//go:generate bash -c "jq .abi ../../core/packages/contracts/out/BeefyClient.sol/BeefyClient.json | abigen --abi - --type BeefyClient --pkg contracts --out beefy_client.go"
//go:generate bash -c "jq .abi ../../core/packages/contracts/out/InboundChannel.sol/InboundChannel.json | abigen --abi - --type InboundChannel --pkg contracts --out inbound_channel.go"
//go:generate bash -c "jq .abi ../../core/packages/contracts/out/OutboundChannel.sol/OutboundChannel.json | abigen --abi - --type OutboundChannel --pkg contracts --out outbound_channel.go"

package contracts
