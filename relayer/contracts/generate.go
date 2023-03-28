//go:generate bash -c "jq .abi ../../core/packages/contracts/out/OpaqueProof.sol/OpaqueProof.json | abigen --abi - --type OpaqueProof --pkg contracts --out opaque_proof.go"
//go:generate bash -c "jq .abi ../../core/packages/contracts/out/BeefyClient.sol/BeefyClient.json | abigen --abi - --type BeefyClient --pkg contracts --out beefy_client.go"
//go:generate bash -c "jq .abi ../../core/packages/contracts/out/InboundQueue.sol/InboundQueue.json | abigen --abi - --type InboundQueue --pkg contracts --out inbound_queue.go"
//go:generate bash -c "jq .abi ../../core/packages/contracts/out/OutboundQueue.sol/OutboundQueue.json | abigen --abi - --type OutboundQueue --pkg contracts --out outbound_queue.go"

package contracts
