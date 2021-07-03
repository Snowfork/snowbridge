---
layout: default
title: Light Client Data Types and Proof Verification
nav_order: 5
permalink: /concepts/polkadot-light-client-verifier/data-types-and-proof-verification
parent: Polkadot Light Client Verifier
grand_parent: Concepts and Architecture
---
# Protocol data types

According to the Beefy protocol, a `Commitment` is signed by validators:

```Solidity
struct Commitment {
    bytes32 payload;
    uint64 blockNumber;
    uint32 validatorSetId;
}
```

The `payload` of that `Commitment` is the latest MMR root hash.

The MMR has the following `MmrLeaf` leaves:

```Rust
/// A leaf that gets added every block to the MMR constructed by [pallet_mmr].
#[derive(RuntimeDebug, PartialEq, Eq, Clone, Encode, Decode)]
pub struct MmrLeaf<BlockNumber, Hash, MerkleRoot> {
	/// Current block parent number and hash.
	pub parent_number_and_hash: (BlockNumber, Hash),
	/// A merkle root of all registered parachain heads.
	pub parachain_heads: MerkleRoot,
	/// A merkle root of the next BEEFY authority set.
	pub beefy_next_authority_set: BeefyNextAuthoritySet<MerkleRoot>,
}

/// Details of the next BEEFY authority set.
#[derive(RuntimeDebug, Default, PartialEq, Eq, Clone, Encode, Decode)]
pub struct BeefyNextAuthoritySet<MerkleRoot> {
	/// Id of the next set.
	///
	/// Id is required to correlate BEEFY signed commitments with the validator set.
	/// Light Client can easily verify that the commitment witness it is getting is
	/// produced by the latest validator set.
	pub id: ValidatorSetId,
	/// Number of validators in the set.
	///
	/// Some BEEFY Light Clients may use an interactive protocol to verify only subset
	/// of signatures. We put set length here, so that these clients can verify the minimal
	/// number of required signatures.
	pub len: u32,
	/// Merkle Root Hash build from BEEFY AuthorityIds.
	///
	/// This is used by Light Clients to confirm that the commitments are signed by the correct
	/// validator set. Light Clients using interactive protocol, might verify only subset of
	/// signatures, hence don't require the full list here (will receive inclusion proofs).
	pub root: MerkleRoot,
}

/// The block number type used by Polkadot.
/// 32-bits will allow for 136 years of blocks assuming 1 block per second.
pub type BlockNumber = u32;

/// A hash of some data used by the relay chain.
pub type Hash = sp_core::H256;

/// A typedef for validator set id.
pub type ValidatorSetId = u64;
```

Messages are abi encoded, hashed and committed in the parachain as an auxiliary digest item.

# Proof verification

To send messages via a channel from the parachain to Ethereum, a proof must be provided. For proof verification, the following is needed on the Ethereum channel:

```Solidity
function submit(
	Message[] calldata _messages,
	OwnParachainHeadPartial _ownParachainHeadPartial, // <-
	bytes32[] memory _parachainHeadsProof, // <-
	BeefyMMRLeafPartial _beefyMMRLeafPartial, // <-
	uint256 _beefyMMRLeafIndex,
	uint256 _beefyMMRLeafCount,
	bytes32[] memory _beefyMMRLeafProof
) public

struct Message {
	address target;
	uint64 nonce;
	bytes payload;
}

struct OwnParachainHeadPartial {
	bytes32 parentHash;
	uint32 number;
	bytes32 stateRoot;
	bytes32 extrinsicsRoot;
}

struct BeefyMMRLeafPartial {
	uint32 parentNumber;
	bytes32 parentHash;
	uint64 nextAuthoritySetId;
	uint32 nextAuthoritySetLen;
	bytes32 nextAuthoritySetRoot;
}
```

The following is performed to verify messages
1. Compute our parachain's message `commitment` by ABI encoding and hashing the `_messages`
2. Compute `ownParachainHead` by hashing the data of the `commitment` together with the contents of `_ownParachainHeadPartial` (see `OwnParachainHeadPartial` above)
3. Compute `parachainHeadsRoot` by verifying the merkle proof using `ownParachainHead` and `_parachainHeadsProof`
4. Compute the `beefyMMRLeaf` using `parachainHeadsRoot` and `_beefyMMRLeafPartial` (see `BeefyMMRLeafPartial` above)
5. Verify inclusion of the beefy MMR leaf in the beefy MMR root using that `beefyMMRLeaf` as well as `_beefyMMRLeafIndex`, `_beefyMMRLeafCount` and `_beefyMMRLeafProof`
