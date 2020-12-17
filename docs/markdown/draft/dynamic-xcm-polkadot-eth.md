Options for dynamic XCM in Polkadot -> Ethereum direction
----------------------------------------------------------------

Option A: Registration/Whitelisting between Parachain+Pallet and Ethereum Smart Contract


+: coupling between pallet and smart contract is explicit. safety is explicit.
-: doesnt allow for dynamic sending, so limits application composability. requires permissiones composition.

--------------------------
Option B: Dynamic/Permissionless Sending

extend commitments pallet:
struct Message {
	address: H160,
	payload: Vec<u8>,
	nonce: u64,
    sourceParachain
}

XCM::Transact::Call::Commitments.add(address, payload_in_rlp)

add {
    determine extrinsic origin and extract parachain identifier from it
    add that to message
}

-> Ethereum
ETHApp is responsible for checking that origin is authorized

+: allows for dynamic sending, no permission required to do xcm to other pallets. makes x-chain composability as unconstrained as smart contract composability
-: no coupling between pallet and smart contract, so safety is left up to the application layer (similar model to ethereum smart contracts)
