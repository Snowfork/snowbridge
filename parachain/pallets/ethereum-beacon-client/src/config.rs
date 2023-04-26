#[cfg(feature = "minimal")]
mod minimal;

#[cfg(not(feature = "minimal"))]
mod mainnet;

#[cfg(feature = "minimal")]
pub use minimal::*;

#[cfg(not(feature = "minimal"))]
pub use mainnet::*;

pub const CURRENT_SYNC_COMMITTEE_INDEX: u64 = 22;
pub const CURRENT_SYNC_COMMITTEE_DEPTH: u64 = 5;

pub const NEXT_SYNC_COMMITTEE_DEPTH: u64 = 5;
pub const NEXT_SYNC_COMMITTEE_INDEX: u64 = 23;

pub const FINALIZED_ROOT_DEPTH: u64 = 6;
pub const FINALIZED_ROOT_INDEX: u64 = 41;

pub const BLOCK_ROOTS_DEPTH: u64 = 5;
pub const BLOCK_ROOTS_INDEX: u64 = 5;

pub const EXECUTION_HEADER_DEPTH: u64 = 4;
pub const EXECUTION_HEADER_INDEX: u64 = 9;

pub const MAX_EXTRA_DATA_BYTES: usize = 32;
pub const MAX_LOGS_BLOOM_SIZE: usize = 256;
pub const MAX_FEE_RECIPIENT_SIZE: usize = 20;

pub const MAX_FINALIZED_HEADER_SLOT_ARRAY: u32 = 1000;
pub const MAX_BRANCH_PROOF_SIZE: u32 = 20;

/// DomainType('0x07000000')
/// https://github.com/ethereum/consensus-specs/blob/dev/specs/altair/beacon-chain.md#domain-types
pub const DOMAIN_SYNC_COMMITTEE: [u8; 4] = [7, 0, 0, 0];

pub const PUBKEY_SIZE: usize = 48;
pub const SIGNATURE_SIZE: usize = 96;
