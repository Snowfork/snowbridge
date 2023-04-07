pub use beefy_client_mock::*;
/// This module was auto-generated with ethers-rs Abigen.
/// More information at: <https://github.com/gakonst/ethers-rs>
#[allow(
    clippy::enum_variant_names,
    clippy::too_many_arguments,
    clippy::upper_case_acronyms,
    clippy::type_complexity,
    dead_code,
    non_camel_case_types,
)]
pub mod beefy_client_mock {
    pub use super::super::shared_types::*;
    #[rustfmt::skip]
    const __ABI: &str = "[{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"randaoCommitDelay\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"randaoCommitExpiration\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"constructor\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"InvalidBitfield\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"InvalidCommitment\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"InvalidMMRLeaf\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"InvalidMMRLeafProof\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"InvalidSignature\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"InvalidTask\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"InvalidValidatorProof\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"NotEnoughClaims\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"PrevRandaoAlreadyCaptured\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"PrevRandaoNotCaptured\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"StaleCommitment\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"TaskExpired\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"WaitPeriodNotOver\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"bytes32\",\"name\":\"mmrRoot\",\"type\":\"bytes32\",\"components\":[],\"indexed\":false},{\"internalType\":\"uint64\",\"name\":\"blockNumber\",\"type\":\"uint64\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"NewMMRRoot\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"previousOwner\",\"type\":\"address\",\"components\":[],\"indexed\":true},{\"internalType\":\"address\",\"name\":\"newOwner\",\"type\":\"address\",\"components\":[],\"indexed\":true}],\"type\":\"event\",\"name\":\"OwnershipTransferred\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[{\"internalType\":\"bytes32\",\"name\":\"commitmentHash\",\"type\":\"bytes32\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"commitPrevRandao\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"bytes32\",\"name\":\"commitmentHash\",\"type\":\"bytes32\",\"components\":[]},{\"internalType\":\"uint256[]\",\"name\":\"bitfield\",\"type\":\"uint256[]\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"createFinalBitfield\",\"outputs\":[{\"internalType\":\"uint256[]\",\"name\":\"\",\"type\":\"uint256[]\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"uint256[]\",\"name\":\"bitsToSet\",\"type\":\"uint256[]\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"length\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"pure\",\"type\":\"function\",\"name\":\"createInitialBitfield\",\"outputs\":[{\"internalType\":\"uint256[]\",\"name\":\"\",\"type\":\"uint256[]\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"currentValidatorSet\",\"outputs\":[{\"internalType\":\"uint128\",\"name\":\"id\",\"type\":\"uint128\",\"components\":[]},{\"internalType\":\"uint128\",\"name\":\"length\",\"type\":\"uint128\",\"components\":[]},{\"internalType\":\"bytes32\",\"name\":\"root\",\"type\":\"bytes32\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"struct BeefyClient.Commitment\",\"name\":\"commitment\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint32\",\"name\":\"blockNumber\",\"type\":\"uint32\",\"components\":[]},{\"internalType\":\"uint64\",\"name\":\"validatorSetID\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"struct BeefyClient.Payload\",\"name\":\"payload\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"bytes32\",\"name\":\"mmrRootHash\",\"type\":\"bytes32\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"prefix\",\"type\":\"bytes\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"suffix\",\"type\":\"bytes\",\"components\":[]}]}]}],\"stateMutability\":\"pure\",\"type\":\"function\",\"name\":\"encodeCommitment_public\",\"outputs\":[{\"internalType\":\"bytes\",\"name\":\"\",\"type\":\"bytes\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"uint64\",\"name\":\"_initialBeefyBlock\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"struct BeefyClient.ValidatorSet\",\"name\":\"_initialValidatorSet\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint128\",\"name\":\"id\",\"type\":\"uint128\",\"components\":[]},{\"internalType\":\"uint128\",\"name\":\"length\",\"type\":\"uint128\",\"components\":[]},{\"internalType\":\"bytes32\",\"name\":\"root\",\"type\":\"bytes32\",\"components\":[]}]},{\"internalType\":\"struct BeefyClient.ValidatorSet\",\"name\":\"_nextValidatorSet\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint128\",\"name\":\"id\",\"type\":\"uint128\",\"components\":[]},{\"internalType\":\"uint128\",\"name\":\"length\",\"type\":\"uint128\",\"components\":[]},{\"internalType\":\"bytes32\",\"name\":\"root\",\"type\":\"bytes32\",\"components\":[]}]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"initialize\",\"outputs\":[]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"latestBeefyBlock\",\"outputs\":[{\"internalType\":\"uint64\",\"name\":\"\",\"type\":\"uint64\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"latestMMRRoot\",\"outputs\":[{\"internalType\":\"bytes32\",\"name\":\"\",\"type\":\"bytes32\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"validatorSetLen\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"pure\",\"type\":\"function\",\"name\":\"minimumSignatureThreshold_public\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"nextValidatorSet\",\"outputs\":[{\"internalType\":\"uint128\",\"name\":\"id\",\"type\":\"uint128\",\"components\":[]},{\"internalType\":\"uint128\",\"name\":\"length\",\"type\":\"uint128\",\"components\":[]},{\"internalType\":\"bytes32\",\"name\":\"root\",\"type\":\"bytes32\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"owner\",\"outputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"randaoCommitDelay\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"randaoCommitExpiration\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"renounceOwnership\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"struct BeefyClient.Commitment\",\"name\":\"commitment\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint32\",\"name\":\"blockNumber\",\"type\":\"uint32\",\"components\":[]},{\"internalType\":\"uint64\",\"name\":\"validatorSetID\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"struct BeefyClient.Payload\",\"name\":\"payload\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"bytes32\",\"name\":\"mmrRootHash\",\"type\":\"bytes32\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"prefix\",\"type\":\"bytes\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"suffix\",\"type\":\"bytes\",\"components\":[]}]}]},{\"internalType\":\"uint256[]\",\"name\":\"bitfield\",\"type\":\"uint256[]\",\"components\":[]},{\"internalType\":\"struct BeefyClient.ValidatorProof[]\",\"name\":\"proofs\",\"type\":\"tuple[]\",\"components\":[{\"internalType\":\"uint8\",\"name\":\"v\",\"type\":\"uint8\",\"components\":[]},{\"internalType\":\"bytes32\",\"name\":\"r\",\"type\":\"bytes32\",\"components\":[]},{\"internalType\":\"bytes32\",\"name\":\"s\",\"type\":\"bytes32\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"index\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"bytes32[]\",\"name\":\"proof\",\"type\":\"bytes32[]\",\"components\":[]}]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"submitFinal\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"struct BeefyClient.Commitment\",\"name\":\"commitment\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint32\",\"name\":\"blockNumber\",\"type\":\"uint32\",\"components\":[]},{\"internalType\":\"uint64\",\"name\":\"validatorSetID\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"struct BeefyClient.Payload\",\"name\":\"payload\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"bytes32\",\"name\":\"mmrRootHash\",\"type\":\"bytes32\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"prefix\",\"type\":\"bytes\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"suffix\",\"type\":\"bytes\",\"components\":[]}]}]},{\"internalType\":\"uint256[]\",\"name\":\"bitfield\",\"type\":\"uint256[]\",\"components\":[]},{\"internalType\":\"struct BeefyClient.ValidatorProof[]\",\"name\":\"proofs\",\"type\":\"tuple[]\",\"components\":[{\"internalType\":\"uint8\",\"name\":\"v\",\"type\":\"uint8\",\"components\":[]},{\"internalType\":\"bytes32\",\"name\":\"r\",\"type\":\"bytes32\",\"components\":[]},{\"internalType\":\"bytes32\",\"name\":\"s\",\"type\":\"bytes32\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"index\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"bytes32[]\",\"name\":\"proof\",\"type\":\"bytes32[]\",\"components\":[]}]},{\"internalType\":\"struct BeefyClient.MMRLeaf\",\"name\":\"leaf\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint8\",\"name\":\"version\",\"type\":\"uint8\",\"components\":[]},{\"internalType\":\"uint32\",\"name\":\"parentNumber\",\"type\":\"uint32\",\"components\":[]},{\"internalType\":\"bytes32\",\"name\":\"parentHash\",\"type\":\"bytes32\",\"components\":[]},{\"internalType\":\"uint64\",\"name\":\"nextAuthoritySetID\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"uint32\",\"name\":\"nextAuthoritySetLen\",\"type\":\"uint32\",\"components\":[]},{\"internalType\":\"bytes32\",\"name\":\"nextAuthoritySetRoot\",\"type\":\"bytes32\",\"components\":[]},{\"internalType\":\"bytes32\",\"name\":\"parachainHeadsRoot\",\"type\":\"bytes32\",\"components\":[]}]},{\"internalType\":\"bytes32[]\",\"name\":\"leafProof\",\"type\":\"bytes32[]\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"leafProofOrder\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"submitFinalWithHandover\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"bytes32\",\"name\":\"commitmentHash\",\"type\":\"bytes32\",\"components\":[]},{\"internalType\":\"uint256[]\",\"name\":\"bitfield\",\"type\":\"uint256[]\",\"components\":[]},{\"internalType\":\"struct BeefyClient.ValidatorProof\",\"name\":\"proof\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint8\",\"name\":\"v\",\"type\":\"uint8\",\"components\":[]},{\"internalType\":\"bytes32\",\"name\":\"r\",\"type\":\"bytes32\",\"components\":[]},{\"internalType\":\"bytes32\",\"name\":\"s\",\"type\":\"bytes32\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"index\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"bytes32[]\",\"name\":\"proof\",\"type\":\"bytes32[]\",\"components\":[]}]}],\"stateMutability\":\"payable\",\"type\":\"function\",\"name\":\"submitInitial\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"bytes32\",\"name\":\"commitmentHash\",\"type\":\"bytes32\",\"components\":[]},{\"internalType\":\"uint256[]\",\"name\":\"bitfield\",\"type\":\"uint256[]\",\"components\":[]},{\"internalType\":\"struct BeefyClient.ValidatorProof\",\"name\":\"proof\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint8\",\"name\":\"v\",\"type\":\"uint8\",\"components\":[]},{\"internalType\":\"bytes32\",\"name\":\"r\",\"type\":\"bytes32\",\"components\":[]},{\"internalType\":\"bytes32\",\"name\":\"s\",\"type\":\"bytes32\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"index\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"bytes32[]\",\"name\":\"proof\",\"type\":\"bytes32[]\",\"components\":[]}]}],\"stateMutability\":\"payable\",\"type\":\"function\",\"name\":\"submitInitialWithHandover\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"bytes32\",\"name\":\"\",\"type\":\"bytes32\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"tasks\",\"outputs\":[{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"uint64\",\"name\":\"blockNumber\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"uint32\",\"name\":\"validatorSetLen\",\"type\":\"uint32\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"prevRandao\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"bytes32\",\"name\":\"bitfieldHash\",\"type\":\"bytes32\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"newOwner\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"transferOwnership\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"bytes32\",\"name\":\"leafHash\",\"type\":\"bytes32\",\"components\":[]},{\"internalType\":\"bytes32[]\",\"name\":\"proof\",\"type\":\"bytes32[]\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"proofOrder\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"verifyMMRLeafProof\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\",\"components\":[]}]}]";
    ///The parsed JSON ABI of the contract.
    pub static BEEFYCLIENTMOCK_ABI: ::ethers::contract::Lazy<::ethers::core::abi::Abi> = ::ethers::contract::Lazy::new(||
    ::ethers::core::utils::__serde_json::from_str(__ABI).expect("ABI is always valid"));
    pub struct BeefyClientMock<M>(::ethers::contract::Contract<M>);
    impl<M> ::core::clone::Clone for BeefyClientMock<M> {
        fn clone(&self) -> Self {
            Self(::core::clone::Clone::clone(&self.0))
        }
    }
    impl<M> ::core::ops::Deref for BeefyClientMock<M> {
        type Target = ::ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> ::core::ops::DerefMut for BeefyClientMock<M> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }
    impl<M> ::core::fmt::Debug for BeefyClientMock<M> {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            f.debug_tuple(stringify!(BeefyClientMock)).field(&self.address()).finish()
        }
    }
    impl<M: ::ethers::providers::Middleware> BeefyClientMock<M> {
        /// Creates a new contract instance with the specified `ethers` client at
        /// `address`. The contract derefs to a `ethers::Contract` object.
        pub fn new<T: Into<::ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            Self(
                ::ethers::contract::Contract::new(
                    address.into(),
                    BEEFYCLIENTMOCK_ABI.clone(),
                    client,
                ),
            )
        }
        ///Calls the contract's `commitPrevRandao` (0xa77cf3d2) function
        pub fn commit_prev_randao(
            &self,
            commitment_hash: [u8; 32],
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([167, 124, 243, 210], commitment_hash)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `createFinalBitfield` (0x8ab81d13) function
        pub fn create_final_bitfield(
            &self,
            commitment_hash: [u8; 32],
            bitfield: ::std::vec::Vec<::ethers::core::types::U256>,
        ) -> ::ethers::contract::builders::ContractCall<
            M,
            ::std::vec::Vec<::ethers::core::types::U256>,
        > {
            self.0
                .method_hash([138, 184, 29, 19], (commitment_hash, bitfield))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `createInitialBitfield` (0x5da57fe9) function
        pub fn create_initial_bitfield(
            &self,
            bits_to_set: ::std::vec::Vec<::ethers::core::types::U256>,
            length: ::ethers::core::types::U256,
        ) -> ::ethers::contract::builders::ContractCall<
            M,
            ::std::vec::Vec<::ethers::core::types::U256>,
        > {
            self.0
                .method_hash([93, 165, 127, 233], (bits_to_set, length))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `currentValidatorSet` (0x2cdea717) function
        pub fn current_validator_set(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, (u128, u128, [u8; 32])> {
            self.0
                .method_hash([44, 222, 167, 23], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `encodeCommitment_public` (0xcc2e015f) function
        pub fn encode_commitment_public(
            &self,
            commitment: Commitment,
        ) -> ::ethers::contract::builders::ContractCall<
            M,
            ::ethers::core::types::Bytes,
        > {
            self.0
                .method_hash([204, 46, 1, 95], (commitment,))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `initialize` (0xe104815d) function
        pub fn initialize(
            &self,
            initial_beefy_block: u64,
            initial_validator_set: ValidatorSet,
            next_validator_set: ValidatorSet,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash(
                    [225, 4, 129, 93],
                    (initial_beefy_block, initial_validator_set, next_validator_set),
                )
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `latestBeefyBlock` (0x66ae69a0) function
        pub fn latest_beefy_block(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, u64> {
            self.0
                .method_hash([102, 174, 105, 160], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `latestMMRRoot` (0x41c9634e) function
        pub fn latest_mmr_root(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, [u8; 32]> {
            self.0
                .method_hash([65, 201, 99, 78], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `minimumSignatureThreshold_public` (0x1eb83603) function
        pub fn minimum_signature_threshold_public(
            &self,
            validator_set_len: ::ethers::core::types::U256,
        ) -> ::ethers::contract::builders::ContractCall<M, ::ethers::core::types::U256> {
            self.0
                .method_hash([30, 184, 54, 3], validator_set_len)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `nextValidatorSet` (0x36667513) function
        pub fn next_validator_set(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, (u128, u128, [u8; 32])> {
            self.0
                .method_hash([54, 102, 117, 19], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `owner` (0x8da5cb5b) function
        pub fn owner(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<
            M,
            ::ethers::core::types::Address,
        > {
            self.0
                .method_hash([141, 165, 203, 91], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `randaoCommitDelay` (0x591d99ee) function
        pub fn randao_commit_delay(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, ::ethers::core::types::U256> {
            self.0
                .method_hash([89, 29, 153, 238], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `randaoCommitExpiration` (0xad209a9b) function
        pub fn randao_commit_expiration(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, ::ethers::core::types::U256> {
            self.0
                .method_hash([173, 32, 154, 155], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `renounceOwnership` (0x715018a6) function
        pub fn renounce_ownership(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([113, 80, 24, 166], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `submitFinal` (0xc46af85d) function
        pub fn submit_final(
            &self,
            commitment: Commitment,
            bitfield: ::std::vec::Vec<::ethers::core::types::U256>,
            proofs: ::std::vec::Vec<ValidatorProof>,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([196, 106, 248, 93], (commitment, bitfield, proofs))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `submitFinalWithHandover` (0x8114d5e8) function
        pub fn submit_final_with_handover(
            &self,
            commitment: Commitment,
            bitfield: ::std::vec::Vec<::ethers::core::types::U256>,
            proofs: ::std::vec::Vec<ValidatorProof>,
            leaf: Mmrleaf,
            leaf_proof: ::std::vec::Vec<[u8; 32]>,
            leaf_proof_order: ::ethers::core::types::U256,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash(
                    [129, 20, 213, 232],
                    (commitment, bitfield, proofs, leaf, leaf_proof, leaf_proof_order),
                )
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `submitInitial` (0x06a9eff7) function
        pub fn submit_initial(
            &self,
            commitment_hash: [u8; 32],
            bitfield: ::std::vec::Vec<::ethers::core::types::U256>,
            proof: ValidatorProof,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([6, 169, 239, 247], (commitment_hash, bitfield, proof))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `submitInitialWithHandover` (0x61de6237) function
        pub fn submit_initial_with_handover(
            &self,
            commitment_hash: [u8; 32],
            bitfield: ::std::vec::Vec<::ethers::core::types::U256>,
            proof: ValidatorProof,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([97, 222, 98, 55], (commitment_hash, bitfield, proof))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `tasks` (0xe579f500) function
        pub fn tasks(
            &self,
            p0: [u8; 32],
        ) -> ::ethers::contract::builders::ContractCall<
            M,
            (
                ::ethers::core::types::Address,
                u64,
                u32,
                ::ethers::core::types::U256,
                [u8; 32],
            ),
        > {
            self.0
                .method_hash([229, 121, 245, 0], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `transferOwnership` (0xf2fde38b) function
        pub fn transfer_ownership(
            &self,
            new_owner: ::ethers::core::types::Address,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([242, 253, 227, 139], new_owner)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `verifyMMRLeafProof` (0xa401662b) function
        pub fn verify_mmr_leaf_proof(
            &self,
            leaf_hash: [u8; 32],
            proof: ::std::vec::Vec<[u8; 32]>,
            proof_order: ::ethers::core::types::U256,
        ) -> ::ethers::contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([164, 1, 102, 43], (leaf_hash, proof, proof_order))
                .expect("method not found (this should never happen)")
        }
        ///Gets the contract's `NewMMRRoot` event
        pub fn new_mmr_root_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            NewMMRRootFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `OwnershipTransferred` event
        pub fn ownership_transferred_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            OwnershipTransferredFilter,
        > {
            self.0.event()
        }
        /// Returns an `Event` builder for all the events of this contract.
        pub fn events(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            BeefyClientMockEvents,
        > {
            self.0.event_with_filter(::core::default::Default::default())
        }
    }
    impl<M: ::ethers::providers::Middleware> From<::ethers::contract::Contract<M>>
    for BeefyClientMock<M> {
        fn from(contract: ::ethers::contract::Contract<M>) -> Self {
            Self::new(contract.address(), contract.client())
        }
    }
    ///Custom Error type `InvalidBitfield` with signature `InvalidBitfield()` and selector `0x6768c0aa`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(name = "InvalidBitfield", abi = "InvalidBitfield()")]
    pub struct InvalidBitfield;
    ///Custom Error type `InvalidCommitment` with signature `InvalidCommitment()` and selector `0xc06789fa`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(name = "InvalidCommitment", abi = "InvalidCommitment()")]
    pub struct InvalidCommitment;
    ///Custom Error type `InvalidMMRLeaf` with signature `InvalidMMRLeaf()` and selector `0xc72c8200`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(name = "InvalidMMRLeaf", abi = "InvalidMMRLeaf()")]
    pub struct InvalidMMRLeaf;
    ///Custom Error type `InvalidMMRLeafProof` with signature `InvalidMMRLeafProof()` and selector `0x128597bb`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(name = "InvalidMMRLeafProof", abi = "InvalidMMRLeafProof()")]
    pub struct InvalidMMRLeafProof;
    ///Custom Error type `InvalidSignature` with signature `InvalidSignature()` and selector `0x8baa579f`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(name = "InvalidSignature", abi = "InvalidSignature()")]
    pub struct InvalidSignature;
    ///Custom Error type `InvalidTask` with signature `InvalidTask()` and selector `0x0531bbb1`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(name = "InvalidTask", abi = "InvalidTask()")]
    pub struct InvalidTask;
    ///Custom Error type `InvalidValidatorProof` with signature `InvalidValidatorProof()` and selector `0xe00153fa`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(name = "InvalidValidatorProof", abi = "InvalidValidatorProof()")]
    pub struct InvalidValidatorProof;
    ///Custom Error type `NotEnoughClaims` with signature `NotEnoughClaims()` and selector `0xee3e74af`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(name = "NotEnoughClaims", abi = "NotEnoughClaims()")]
    pub struct NotEnoughClaims;
    ///Custom Error type `PrevRandaoAlreadyCaptured` with signature `PrevRandaoAlreadyCaptured()` and selector `0xe31d9005`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(name = "PrevRandaoAlreadyCaptured", abi = "PrevRandaoAlreadyCaptured()")]
    pub struct PrevRandaoAlreadyCaptured;
    ///Custom Error type `PrevRandaoNotCaptured` with signature `PrevRandaoNotCaptured()` and selector `0x78ef3a47`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(name = "PrevRandaoNotCaptured", abi = "PrevRandaoNotCaptured()")]
    pub struct PrevRandaoNotCaptured;
    ///Custom Error type `StaleCommitment` with signature `StaleCommitment()` and selector `0x3d618e50`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(name = "StaleCommitment", abi = "StaleCommitment()")]
    pub struct StaleCommitment;
    ///Custom Error type `TaskExpired` with signature `TaskExpired()` and selector `0xc0343103`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(name = "TaskExpired", abi = "TaskExpired()")]
    pub struct TaskExpired;
    ///Custom Error type `WaitPeriodNotOver` with signature `WaitPeriodNotOver()` and selector `0xc77c1949`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(name = "WaitPeriodNotOver", abi = "WaitPeriodNotOver()")]
    pub struct WaitPeriodNotOver;
    ///Container type for all of the contract's custom errors
    #[derive(Clone, ::ethers::contract::EthAbiType, Debug, PartialEq, Eq, Hash)]
    pub enum BeefyClientMockErrors {
        InvalidBitfield(InvalidBitfield),
        InvalidCommitment(InvalidCommitment),
        InvalidMMRLeaf(InvalidMMRLeaf),
        InvalidMMRLeafProof(InvalidMMRLeafProof),
        InvalidSignature(InvalidSignature),
        InvalidTask(InvalidTask),
        InvalidValidatorProof(InvalidValidatorProof),
        NotEnoughClaims(NotEnoughClaims),
        PrevRandaoAlreadyCaptured(PrevRandaoAlreadyCaptured),
        PrevRandaoNotCaptured(PrevRandaoNotCaptured),
        StaleCommitment(StaleCommitment),
        TaskExpired(TaskExpired),
        WaitPeriodNotOver(WaitPeriodNotOver),
    }
    impl ::ethers::core::abi::AbiDecode for BeefyClientMockErrors {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::core::result::Result<Self, ::ethers::core::abi::AbiError> {
            let data = data.as_ref();
            if let Ok(decoded)
                = <InvalidBitfield as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::InvalidBitfield(decoded));
            }
            if let Ok(decoded)
                = <InvalidCommitment as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::InvalidCommitment(decoded));
            }
            if let Ok(decoded)
                = <InvalidMMRLeaf as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::InvalidMMRLeaf(decoded));
            }
            if let Ok(decoded)
                = <InvalidMMRLeafProof as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::InvalidMMRLeafProof(decoded));
            }
            if let Ok(decoded)
                = <InvalidSignature as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::InvalidSignature(decoded));
            }
            if let Ok(decoded)
                = <InvalidTask as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::InvalidTask(decoded));
            }
            if let Ok(decoded)
                = <InvalidValidatorProof as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::InvalidValidatorProof(decoded));
            }
            if let Ok(decoded)
                = <NotEnoughClaims as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::NotEnoughClaims(decoded));
            }
            if let Ok(decoded)
                = <PrevRandaoAlreadyCaptured as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::PrevRandaoAlreadyCaptured(decoded));
            }
            if let Ok(decoded)
                = <PrevRandaoNotCaptured as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::PrevRandaoNotCaptured(decoded));
            }
            if let Ok(decoded)
                = <StaleCommitment as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::StaleCommitment(decoded));
            }
            if let Ok(decoded)
                = <TaskExpired as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::TaskExpired(decoded));
            }
            if let Ok(decoded)
                = <WaitPeriodNotOver as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::WaitPeriodNotOver(decoded));
            }
            Err(::ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ::ethers::core::abi::AbiEncode for BeefyClientMockErrors {
        fn encode(self) -> ::std::vec::Vec<u8> {
            match self {
                Self::InvalidBitfield(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::InvalidCommitment(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::InvalidMMRLeaf(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::InvalidMMRLeafProof(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::InvalidSignature(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::InvalidTask(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::InvalidValidatorProof(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::NotEnoughClaims(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::PrevRandaoAlreadyCaptured(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::PrevRandaoNotCaptured(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::StaleCommitment(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::TaskExpired(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::WaitPeriodNotOver(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
            }
        }
    }
    impl ::core::fmt::Display for BeefyClientMockErrors {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::InvalidBitfield(element) => ::core::fmt::Display::fmt(element, f),
                Self::InvalidCommitment(element) => ::core::fmt::Display::fmt(element, f),
                Self::InvalidMMRLeaf(element) => ::core::fmt::Display::fmt(element, f),
                Self::InvalidMMRLeafProof(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::InvalidSignature(element) => ::core::fmt::Display::fmt(element, f),
                Self::InvalidTask(element) => ::core::fmt::Display::fmt(element, f),
                Self::InvalidValidatorProof(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::NotEnoughClaims(element) => ::core::fmt::Display::fmt(element, f),
                Self::PrevRandaoAlreadyCaptured(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::PrevRandaoNotCaptured(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::StaleCommitment(element) => ::core::fmt::Display::fmt(element, f),
                Self::TaskExpired(element) => ::core::fmt::Display::fmt(element, f),
                Self::WaitPeriodNotOver(element) => ::core::fmt::Display::fmt(element, f),
            }
        }
    }
    impl ::core::convert::From<InvalidBitfield> for BeefyClientMockErrors {
        fn from(value: InvalidBitfield) -> Self {
            Self::InvalidBitfield(value)
        }
    }
    impl ::core::convert::From<InvalidCommitment> for BeefyClientMockErrors {
        fn from(value: InvalidCommitment) -> Self {
            Self::InvalidCommitment(value)
        }
    }
    impl ::core::convert::From<InvalidMMRLeaf> for BeefyClientMockErrors {
        fn from(value: InvalidMMRLeaf) -> Self {
            Self::InvalidMMRLeaf(value)
        }
    }
    impl ::core::convert::From<InvalidMMRLeafProof> for BeefyClientMockErrors {
        fn from(value: InvalidMMRLeafProof) -> Self {
            Self::InvalidMMRLeafProof(value)
        }
    }
    impl ::core::convert::From<InvalidSignature> for BeefyClientMockErrors {
        fn from(value: InvalidSignature) -> Self {
            Self::InvalidSignature(value)
        }
    }
    impl ::core::convert::From<InvalidTask> for BeefyClientMockErrors {
        fn from(value: InvalidTask) -> Self {
            Self::InvalidTask(value)
        }
    }
    impl ::core::convert::From<InvalidValidatorProof> for BeefyClientMockErrors {
        fn from(value: InvalidValidatorProof) -> Self {
            Self::InvalidValidatorProof(value)
        }
    }
    impl ::core::convert::From<NotEnoughClaims> for BeefyClientMockErrors {
        fn from(value: NotEnoughClaims) -> Self {
            Self::NotEnoughClaims(value)
        }
    }
    impl ::core::convert::From<PrevRandaoAlreadyCaptured> for BeefyClientMockErrors {
        fn from(value: PrevRandaoAlreadyCaptured) -> Self {
            Self::PrevRandaoAlreadyCaptured(value)
        }
    }
    impl ::core::convert::From<PrevRandaoNotCaptured> for BeefyClientMockErrors {
        fn from(value: PrevRandaoNotCaptured) -> Self {
            Self::PrevRandaoNotCaptured(value)
        }
    }
    impl ::core::convert::From<StaleCommitment> for BeefyClientMockErrors {
        fn from(value: StaleCommitment) -> Self {
            Self::StaleCommitment(value)
        }
    }
    impl ::core::convert::From<TaskExpired> for BeefyClientMockErrors {
        fn from(value: TaskExpired) -> Self {
            Self::TaskExpired(value)
        }
    }
    impl ::core::convert::From<WaitPeriodNotOver> for BeefyClientMockErrors {
        fn from(value: WaitPeriodNotOver) -> Self {
            Self::WaitPeriodNotOver(value)
        }
    }
    #[derive(
        Clone,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethevent(name = "NewMMRRoot", abi = "NewMMRRoot(bytes32,uint64)")]
    pub struct NewMMRRootFilter {
        pub mmr_root: [u8; 32],
        pub block_number: u64,
    }
    #[derive(
        Clone,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethevent(
        name = "OwnershipTransferred",
        abi = "OwnershipTransferred(address,address)"
    )]
    pub struct OwnershipTransferredFilter {
        #[ethevent(indexed)]
        pub previous_owner: ::ethers::core::types::Address,
        #[ethevent(indexed)]
        pub new_owner: ::ethers::core::types::Address,
    }
    ///Container type for all of the contract's events
    #[derive(Clone, ::ethers::contract::EthAbiType, Debug, PartialEq, Eq, Hash)]
    pub enum BeefyClientMockEvents {
        NewMMRRootFilter(NewMMRRootFilter),
        OwnershipTransferredFilter(OwnershipTransferredFilter),
    }
    impl ::ethers::contract::EthLogDecode for BeefyClientMockEvents {
        fn decode_log(
            log: &::ethers::core::abi::RawLog,
        ) -> ::core::result::Result<Self, ::ethers::core::abi::Error> {
            if let Ok(decoded) = NewMMRRootFilter::decode_log(log) {
                return Ok(BeefyClientMockEvents::NewMMRRootFilter(decoded));
            }
            if let Ok(decoded) = OwnershipTransferredFilter::decode_log(log) {
                return Ok(BeefyClientMockEvents::OwnershipTransferredFilter(decoded));
            }
            Err(::ethers::core::abi::Error::InvalidData)
        }
    }
    impl ::core::fmt::Display for BeefyClientMockEvents {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::NewMMRRootFilter(element) => ::core::fmt::Display::fmt(element, f),
                Self::OwnershipTransferredFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
            }
        }
    }
    impl ::core::convert::From<NewMMRRootFilter> for BeefyClientMockEvents {
        fn from(value: NewMMRRootFilter) -> Self {
            Self::NewMMRRootFilter(value)
        }
    }
    impl ::core::convert::From<OwnershipTransferredFilter> for BeefyClientMockEvents {
        fn from(value: OwnershipTransferredFilter) -> Self {
            Self::OwnershipTransferredFilter(value)
        }
    }
    ///Container type for all input parameters for the `commitPrevRandao` function with signature `commitPrevRandao(bytes32)` and selector `0xa77cf3d2`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "commitPrevRandao", abi = "commitPrevRandao(bytes32)")]
    pub struct CommitPrevRandaoCall {
        pub commitment_hash: [u8; 32],
    }
    ///Container type for all input parameters for the `createFinalBitfield` function with signature `createFinalBitfield(bytes32,uint256[])` and selector `0x8ab81d13`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(
        name = "createFinalBitfield",
        abi = "createFinalBitfield(bytes32,uint256[])"
    )]
    pub struct CreateFinalBitfieldCall {
        pub commitment_hash: [u8; 32],
        pub bitfield: ::std::vec::Vec<::ethers::core::types::U256>,
    }
    ///Container type for all input parameters for the `createInitialBitfield` function with signature `createInitialBitfield(uint256[],uint256)` and selector `0x5da57fe9`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(
        name = "createInitialBitfield",
        abi = "createInitialBitfield(uint256[],uint256)"
    )]
    pub struct CreateInitialBitfieldCall {
        pub bits_to_set: ::std::vec::Vec<::ethers::core::types::U256>,
        pub length: ::ethers::core::types::U256,
    }
    ///Container type for all input parameters for the `currentValidatorSet` function with signature `currentValidatorSet()` and selector `0x2cdea717`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "currentValidatorSet", abi = "currentValidatorSet()")]
    pub struct CurrentValidatorSetCall;
    ///Container type for all input parameters for the `encodeCommitment_public` function with signature `encodeCommitment_public((uint32,uint64,(bytes32,bytes,bytes)))` and selector `0xcc2e015f`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(
        name = "encodeCommitment_public",
        abi = "encodeCommitment_public((uint32,uint64,(bytes32,bytes,bytes)))"
    )]
    pub struct EncodeCommitmentPublicCall {
        pub commitment: Commitment,
    }
    ///Container type for all input parameters for the `initialize` function with signature `initialize(uint64,(uint128,uint128,bytes32),(uint128,uint128,bytes32))` and selector `0xe104815d`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(
        name = "initialize",
        abi = "initialize(uint64,(uint128,uint128,bytes32),(uint128,uint128,bytes32))"
    )]
    pub struct InitializeCall {
        pub initial_beefy_block: u64,
        pub initial_validator_set: ValidatorSet,
        pub next_validator_set: ValidatorSet,
    }
    ///Container type for all input parameters for the `latestBeefyBlock` function with signature `latestBeefyBlock()` and selector `0x66ae69a0`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "latestBeefyBlock", abi = "latestBeefyBlock()")]
    pub struct LatestBeefyBlockCall;
    ///Container type for all input parameters for the `latestMMRRoot` function with signature `latestMMRRoot()` and selector `0x41c9634e`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "latestMMRRoot", abi = "latestMMRRoot()")]
    pub struct LatestMMRRootCall;
    ///Container type for all input parameters for the `minimumSignatureThreshold_public` function with signature `minimumSignatureThreshold_public(uint256)` and selector `0x1eb83603`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(
        name = "minimumSignatureThreshold_public",
        abi = "minimumSignatureThreshold_public(uint256)"
    )]
    pub struct MinimumSignatureThresholdPublicCall {
        pub validator_set_len: ::ethers::core::types::U256,
    }
    ///Container type for all input parameters for the `nextValidatorSet` function with signature `nextValidatorSet()` and selector `0x36667513`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "nextValidatorSet", abi = "nextValidatorSet()")]
    pub struct NextValidatorSetCall;
    ///Container type for all input parameters for the `owner` function with signature `owner()` and selector `0x8da5cb5b`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "owner", abi = "owner()")]
    pub struct OwnerCall;
    ///Container type for all input parameters for the `randaoCommitDelay` function with signature `randaoCommitDelay()` and selector `0x591d99ee`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "randaoCommitDelay", abi = "randaoCommitDelay()")]
    pub struct RandaoCommitDelayCall;
    ///Container type for all input parameters for the `randaoCommitExpiration` function with signature `randaoCommitExpiration()` and selector `0xad209a9b`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "randaoCommitExpiration", abi = "randaoCommitExpiration()")]
    pub struct RandaoCommitExpirationCall;
    ///Container type for all input parameters for the `renounceOwnership` function with signature `renounceOwnership()` and selector `0x715018a6`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "renounceOwnership", abi = "renounceOwnership()")]
    pub struct RenounceOwnershipCall;
    ///Container type for all input parameters for the `submitFinal` function with signature `submitFinal((uint32,uint64,(bytes32,bytes,bytes)),uint256[],(uint8,bytes32,bytes32,uint256,address,bytes32[])[])` and selector `0xc46af85d`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(
        name = "submitFinal",
        abi = "submitFinal((uint32,uint64,(bytes32,bytes,bytes)),uint256[],(uint8,bytes32,bytes32,uint256,address,bytes32[])[])"
    )]
    pub struct SubmitFinalCall {
        pub commitment: Commitment,
        pub bitfield: ::std::vec::Vec<::ethers::core::types::U256>,
        pub proofs: ::std::vec::Vec<ValidatorProof>,
    }
    ///Container type for all input parameters for the `submitFinalWithHandover` function with signature `submitFinalWithHandover((uint32,uint64,(bytes32,bytes,bytes)),uint256[],(uint8,bytes32,bytes32,uint256,address,bytes32[])[],(uint8,uint32,bytes32,uint64,uint32,bytes32,bytes32),bytes32[],uint256)` and selector `0x8114d5e8`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(
        name = "submitFinalWithHandover",
        abi = "submitFinalWithHandover((uint32,uint64,(bytes32,bytes,bytes)),uint256[],(uint8,bytes32,bytes32,uint256,address,bytes32[])[],(uint8,uint32,bytes32,uint64,uint32,bytes32,bytes32),bytes32[],uint256)"
    )]
    pub struct SubmitFinalWithHandoverCall {
        pub commitment: Commitment,
        pub bitfield: ::std::vec::Vec<::ethers::core::types::U256>,
        pub proofs: ::std::vec::Vec<ValidatorProof>,
        pub leaf: Mmrleaf,
        pub leaf_proof: ::std::vec::Vec<[u8; 32]>,
        pub leaf_proof_order: ::ethers::core::types::U256,
    }
    ///Container type for all input parameters for the `submitInitial` function with signature `submitInitial(bytes32,uint256[],(uint8,bytes32,bytes32,uint256,address,bytes32[]))` and selector `0x06a9eff7`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(
        name = "submitInitial",
        abi = "submitInitial(bytes32,uint256[],(uint8,bytes32,bytes32,uint256,address,bytes32[]))"
    )]
    pub struct SubmitInitialCall {
        pub commitment_hash: [u8; 32],
        pub bitfield: ::std::vec::Vec<::ethers::core::types::U256>,
        pub proof: ValidatorProof,
    }
    ///Container type for all input parameters for the `submitInitialWithHandover` function with signature `submitInitialWithHandover(bytes32,uint256[],(uint8,bytes32,bytes32,uint256,address,bytes32[]))` and selector `0x61de6237`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(
        name = "submitInitialWithHandover",
        abi = "submitInitialWithHandover(bytes32,uint256[],(uint8,bytes32,bytes32,uint256,address,bytes32[]))"
    )]
    pub struct SubmitInitialWithHandoverCall {
        pub commitment_hash: [u8; 32],
        pub bitfield: ::std::vec::Vec<::ethers::core::types::U256>,
        pub proof: ValidatorProof,
    }
    ///Container type for all input parameters for the `tasks` function with signature `tasks(bytes32)` and selector `0xe579f500`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "tasks", abi = "tasks(bytes32)")]
    pub struct TasksCall(pub [u8; 32]);
    ///Container type for all input parameters for the `transferOwnership` function with signature `transferOwnership(address)` and selector `0xf2fde38b`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "transferOwnership", abi = "transferOwnership(address)")]
    pub struct TransferOwnershipCall {
        pub new_owner: ::ethers::core::types::Address,
    }
    ///Container type for all input parameters for the `verifyMMRLeafProof` function with signature `verifyMMRLeafProof(bytes32,bytes32[],uint256)` and selector `0xa401662b`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(
        name = "verifyMMRLeafProof",
        abi = "verifyMMRLeafProof(bytes32,bytes32[],uint256)"
    )]
    pub struct VerifyMMRLeafProofCall {
        pub leaf_hash: [u8; 32],
        pub proof: ::std::vec::Vec<[u8; 32]>,
        pub proof_order: ::ethers::core::types::U256,
    }
    ///Container type for all of the contract's call
    #[derive(Clone, ::ethers::contract::EthAbiType, Debug, PartialEq, Eq, Hash)]
    pub enum BeefyClientMockCalls {
        CommitPrevRandao(CommitPrevRandaoCall),
        CreateFinalBitfield(CreateFinalBitfieldCall),
        CreateInitialBitfield(CreateInitialBitfieldCall),
        CurrentValidatorSet(CurrentValidatorSetCall),
        EncodeCommitmentPublic(EncodeCommitmentPublicCall),
        Initialize(InitializeCall),
        LatestBeefyBlock(LatestBeefyBlockCall),
        LatestMMRRoot(LatestMMRRootCall),
        MinimumSignatureThresholdPublic(MinimumSignatureThresholdPublicCall),
        NextValidatorSet(NextValidatorSetCall),
        Owner(OwnerCall),
        RandaoCommitDelay(RandaoCommitDelayCall),
        RandaoCommitExpiration(RandaoCommitExpirationCall),
        RenounceOwnership(RenounceOwnershipCall),
        SubmitFinal(SubmitFinalCall),
        SubmitFinalWithHandover(SubmitFinalWithHandoverCall),
        SubmitInitial(SubmitInitialCall),
        SubmitInitialWithHandover(SubmitInitialWithHandoverCall),
        Tasks(TasksCall),
        TransferOwnership(TransferOwnershipCall),
        VerifyMMRLeafProof(VerifyMMRLeafProofCall),
    }
    impl ::ethers::core::abi::AbiDecode for BeefyClientMockCalls {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::core::result::Result<Self, ::ethers::core::abi::AbiError> {
            let data = data.as_ref();
            if let Ok(decoded)
                = <CommitPrevRandaoCall as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::CommitPrevRandao(decoded));
            }
            if let Ok(decoded)
                = <CreateFinalBitfieldCall as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::CreateFinalBitfield(decoded));
            }
            if let Ok(decoded)
                = <CreateInitialBitfieldCall as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::CreateInitialBitfield(decoded));
            }
            if let Ok(decoded)
                = <CurrentValidatorSetCall as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::CurrentValidatorSet(decoded));
            }
            if let Ok(decoded)
                = <EncodeCommitmentPublicCall as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::EncodeCommitmentPublic(decoded));
            }
            if let Ok(decoded)
                = <InitializeCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::Initialize(decoded));
            }
            if let Ok(decoded)
                = <LatestBeefyBlockCall as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::LatestBeefyBlock(decoded));
            }
            if let Ok(decoded)
                = <LatestMMRRootCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::LatestMMRRoot(decoded));
            }
            if let Ok(decoded)
                = <MinimumSignatureThresholdPublicCall as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::MinimumSignatureThresholdPublic(decoded));
            }
            if let Ok(decoded)
                = <NextValidatorSetCall as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::NextValidatorSet(decoded));
            }
            if let Ok(decoded)
                = <OwnerCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::Owner(decoded));
            }
            if let Ok(decoded)
                = <RandaoCommitDelayCall as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::RandaoCommitDelay(decoded));
            }
            if let Ok(decoded)
                = <RandaoCommitExpirationCall as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::RandaoCommitExpiration(decoded));
            }
            if let Ok(decoded)
                = <RenounceOwnershipCall as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::RenounceOwnership(decoded));
            }
            if let Ok(decoded)
                = <SubmitFinalCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::SubmitFinal(decoded));
            }
            if let Ok(decoded)
                = <SubmitFinalWithHandoverCall as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::SubmitFinalWithHandover(decoded));
            }
            if let Ok(decoded)
                = <SubmitInitialCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::SubmitInitial(decoded));
            }
            if let Ok(decoded)
                = <SubmitInitialWithHandoverCall as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::SubmitInitialWithHandover(decoded));
            }
            if let Ok(decoded)
                = <TasksCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::Tasks(decoded));
            }
            if let Ok(decoded)
                = <TransferOwnershipCall as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::TransferOwnership(decoded));
            }
            if let Ok(decoded)
                = <VerifyMMRLeafProofCall as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::VerifyMMRLeafProof(decoded));
            }
            Err(::ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ::ethers::core::abi::AbiEncode for BeefyClientMockCalls {
        fn encode(self) -> Vec<u8> {
            match self {
                Self::CommitPrevRandao(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::CreateFinalBitfield(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::CreateInitialBitfield(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::CurrentValidatorSet(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::EncodeCommitmentPublic(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::Initialize(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::LatestBeefyBlock(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::LatestMMRRoot(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::MinimumSignatureThresholdPublic(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::NextValidatorSet(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::Owner(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::RandaoCommitDelay(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::RandaoCommitExpiration(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::RenounceOwnership(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::SubmitFinal(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::SubmitFinalWithHandover(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::SubmitInitial(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::SubmitInitialWithHandover(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::Tasks(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::TransferOwnership(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::VerifyMMRLeafProof(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
            }
        }
    }
    impl ::core::fmt::Display for BeefyClientMockCalls {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::CommitPrevRandao(element) => ::core::fmt::Display::fmt(element, f),
                Self::CreateFinalBitfield(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::CreateInitialBitfield(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::CurrentValidatorSet(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::EncodeCommitmentPublic(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::Initialize(element) => ::core::fmt::Display::fmt(element, f),
                Self::LatestBeefyBlock(element) => ::core::fmt::Display::fmt(element, f),
                Self::LatestMMRRoot(element) => ::core::fmt::Display::fmt(element, f),
                Self::MinimumSignatureThresholdPublic(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::NextValidatorSet(element) => ::core::fmt::Display::fmt(element, f),
                Self::Owner(element) => ::core::fmt::Display::fmt(element, f),
                Self::RandaoCommitDelay(element) => ::core::fmt::Display::fmt(element, f),
                Self::RandaoCommitExpiration(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::RenounceOwnership(element) => ::core::fmt::Display::fmt(element, f),
                Self::SubmitFinal(element) => ::core::fmt::Display::fmt(element, f),
                Self::SubmitFinalWithHandover(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::SubmitInitial(element) => ::core::fmt::Display::fmt(element, f),
                Self::SubmitInitialWithHandover(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::Tasks(element) => ::core::fmt::Display::fmt(element, f),
                Self::TransferOwnership(element) => ::core::fmt::Display::fmt(element, f),
                Self::VerifyMMRLeafProof(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
            }
        }
    }
    impl ::core::convert::From<CommitPrevRandaoCall> for BeefyClientMockCalls {
        fn from(value: CommitPrevRandaoCall) -> Self {
            Self::CommitPrevRandao(value)
        }
    }
    impl ::core::convert::From<CreateFinalBitfieldCall> for BeefyClientMockCalls {
        fn from(value: CreateFinalBitfieldCall) -> Self {
            Self::CreateFinalBitfield(value)
        }
    }
    impl ::core::convert::From<CreateInitialBitfieldCall> for BeefyClientMockCalls {
        fn from(value: CreateInitialBitfieldCall) -> Self {
            Self::CreateInitialBitfield(value)
        }
    }
    impl ::core::convert::From<CurrentValidatorSetCall> for BeefyClientMockCalls {
        fn from(value: CurrentValidatorSetCall) -> Self {
            Self::CurrentValidatorSet(value)
        }
    }
    impl ::core::convert::From<EncodeCommitmentPublicCall> for BeefyClientMockCalls {
        fn from(value: EncodeCommitmentPublicCall) -> Self {
            Self::EncodeCommitmentPublic(value)
        }
    }
    impl ::core::convert::From<InitializeCall> for BeefyClientMockCalls {
        fn from(value: InitializeCall) -> Self {
            Self::Initialize(value)
        }
    }
    impl ::core::convert::From<LatestBeefyBlockCall> for BeefyClientMockCalls {
        fn from(value: LatestBeefyBlockCall) -> Self {
            Self::LatestBeefyBlock(value)
        }
    }
    impl ::core::convert::From<LatestMMRRootCall> for BeefyClientMockCalls {
        fn from(value: LatestMMRRootCall) -> Self {
            Self::LatestMMRRoot(value)
        }
    }
    impl ::core::convert::From<MinimumSignatureThresholdPublicCall>
    for BeefyClientMockCalls {
        fn from(value: MinimumSignatureThresholdPublicCall) -> Self {
            Self::MinimumSignatureThresholdPublic(value)
        }
    }
    impl ::core::convert::From<NextValidatorSetCall> for BeefyClientMockCalls {
        fn from(value: NextValidatorSetCall) -> Self {
            Self::NextValidatorSet(value)
        }
    }
    impl ::core::convert::From<OwnerCall> for BeefyClientMockCalls {
        fn from(value: OwnerCall) -> Self {
            Self::Owner(value)
        }
    }
    impl ::core::convert::From<RandaoCommitDelayCall> for BeefyClientMockCalls {
        fn from(value: RandaoCommitDelayCall) -> Self {
            Self::RandaoCommitDelay(value)
        }
    }
    impl ::core::convert::From<RandaoCommitExpirationCall> for BeefyClientMockCalls {
        fn from(value: RandaoCommitExpirationCall) -> Self {
            Self::RandaoCommitExpiration(value)
        }
    }
    impl ::core::convert::From<RenounceOwnershipCall> for BeefyClientMockCalls {
        fn from(value: RenounceOwnershipCall) -> Self {
            Self::RenounceOwnership(value)
        }
    }
    impl ::core::convert::From<SubmitFinalCall> for BeefyClientMockCalls {
        fn from(value: SubmitFinalCall) -> Self {
            Self::SubmitFinal(value)
        }
    }
    impl ::core::convert::From<SubmitFinalWithHandoverCall> for BeefyClientMockCalls {
        fn from(value: SubmitFinalWithHandoverCall) -> Self {
            Self::SubmitFinalWithHandover(value)
        }
    }
    impl ::core::convert::From<SubmitInitialCall> for BeefyClientMockCalls {
        fn from(value: SubmitInitialCall) -> Self {
            Self::SubmitInitial(value)
        }
    }
    impl ::core::convert::From<SubmitInitialWithHandoverCall> for BeefyClientMockCalls {
        fn from(value: SubmitInitialWithHandoverCall) -> Self {
            Self::SubmitInitialWithHandover(value)
        }
    }
    impl ::core::convert::From<TasksCall> for BeefyClientMockCalls {
        fn from(value: TasksCall) -> Self {
            Self::Tasks(value)
        }
    }
    impl ::core::convert::From<TransferOwnershipCall> for BeefyClientMockCalls {
        fn from(value: TransferOwnershipCall) -> Self {
            Self::TransferOwnership(value)
        }
    }
    impl ::core::convert::From<VerifyMMRLeafProofCall> for BeefyClientMockCalls {
        fn from(value: VerifyMMRLeafProofCall) -> Self {
            Self::VerifyMMRLeafProof(value)
        }
    }
    ///Container type for all return fields from the `createFinalBitfield` function with signature `createFinalBitfield(bytes32,uint256[])` and selector `0x8ab81d13`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct CreateFinalBitfieldReturn(
        pub ::std::vec::Vec<::ethers::core::types::U256>,
    );
    ///Container type for all return fields from the `createInitialBitfield` function with signature `createInitialBitfield(uint256[],uint256)` and selector `0x5da57fe9`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct CreateInitialBitfieldReturn(
        pub ::std::vec::Vec<::ethers::core::types::U256>,
    );
    ///Container type for all return fields from the `currentValidatorSet` function with signature `currentValidatorSet()` and selector `0x2cdea717`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct CurrentValidatorSetReturn {
        pub id: u128,
        pub length: u128,
        pub root: [u8; 32],
    }
    ///Container type for all return fields from the `encodeCommitment_public` function with signature `encodeCommitment_public((uint32,uint64,(bytes32,bytes,bytes)))` and selector `0xcc2e015f`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct EncodeCommitmentPublicReturn(pub ::ethers::core::types::Bytes);
    ///Container type for all return fields from the `latestBeefyBlock` function with signature `latestBeefyBlock()` and selector `0x66ae69a0`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct LatestBeefyBlockReturn(pub u64);
    ///Container type for all return fields from the `latestMMRRoot` function with signature `latestMMRRoot()` and selector `0x41c9634e`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct LatestMMRRootReturn(pub [u8; 32]);
    ///Container type for all return fields from the `minimumSignatureThreshold_public` function with signature `minimumSignatureThreshold_public(uint256)` and selector `0x1eb83603`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct MinimumSignatureThresholdPublicReturn(pub ::ethers::core::types::U256);
    ///Container type for all return fields from the `nextValidatorSet` function with signature `nextValidatorSet()` and selector `0x36667513`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct NextValidatorSetReturn {
        pub id: u128,
        pub length: u128,
        pub root: [u8; 32],
    }
    ///Container type for all return fields from the `owner` function with signature `owner()` and selector `0x8da5cb5b`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct OwnerReturn(pub ::ethers::core::types::Address);
    ///Container type for all return fields from the `randaoCommitDelay` function with signature `randaoCommitDelay()` and selector `0x591d99ee`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct RandaoCommitDelayReturn(pub ::ethers::core::types::U256);
    ///Container type for all return fields from the `randaoCommitExpiration` function with signature `randaoCommitExpiration()` and selector `0xad209a9b`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct RandaoCommitExpirationReturn(pub ::ethers::core::types::U256);
    ///Container type for all return fields from the `tasks` function with signature `tasks(bytes32)` and selector `0xe579f500`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct TasksReturn {
        pub account: ::ethers::core::types::Address,
        pub block_number: u64,
        pub validator_set_len: u32,
        pub prev_randao: ::ethers::core::types::U256,
        pub bitfield_hash: [u8; 32],
    }
    ///Container type for all return fields from the `verifyMMRLeafProof` function with signature `verifyMMRLeafProof(bytes32,bytes32[],uint256)` and selector `0xa401662b`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct VerifyMMRLeafProofReturn(pub bool);
}
