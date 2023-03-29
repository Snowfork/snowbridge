///`Commitment(uint32,uint64,(bytes32,bytes,bytes))`
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
pub struct Commitment {
    pub block_number: u32,
    pub validator_set_id: u64,
    pub payload: Payload,
}
///`Mmrleaf(uint8,uint32,bytes32,uint64,uint32,bytes32,bytes32)`
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
pub struct Mmrleaf {
    pub version: u8,
    pub parent_number: u32,
    pub parent_hash: [u8; 32],
    pub next_authority_set_id: u64,
    pub next_authority_set_len: u32,
    pub next_authority_set_root: [u8; 32],
    pub parachain_heads_root: [u8; 32],
}
///`Payload(bytes32,bytes,bytes)`
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
pub struct Payload {
    pub mmr_root_hash: [u8; 32],
    pub prefix: ::ethers::core::types::Bytes,
    pub suffix: ::ethers::core::types::Bytes,
}
///`ValidatorProof(uint8,bytes32,bytes32,uint256,address,bytes32[])`
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
pub struct ValidatorProof {
    pub v: u8,
    pub r: [u8; 32],
    pub s: [u8; 32],
    pub index: ::ethers::core::types::U256,
    pub account: ::ethers::core::types::Address,
    pub proof: ::std::vec::Vec<[u8; 32]>,
}
///`ValidatorSet(uint128,uint128,bytes32)`
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
pub struct ValidatorSet {
    pub id: u128,
    pub length: u128,
    pub root: [u8; 32],
}
