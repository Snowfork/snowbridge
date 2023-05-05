use static_assertions::const_assert;

pub mod mainnet;
pub mod minimal;

#[cfg(feature = "minimal")]
pub use minimal::*;

#[cfg(not(feature = "minimal"))]
pub use mainnet::*;

pub const CURRENT_SYNC_COMMITTEE_INDEX: usize = 22;
pub const NEXT_SYNC_COMMITTEE_INDEX: usize = 23;
pub const SYNC_COMMITTEE_DEPTH: usize = 5;

pub const FINALIZED_ROOT_DEPTH: usize = 6;
pub const FINALIZED_ROOT_INDEX: usize = 41;

pub const BLOCK_ROOTS_DEPTH: usize = 5;
pub const BLOCK_ROOTS_INDEX: usize = 5;

pub const EXECUTION_HEADER_DEPTH: usize = 4;
pub const EXECUTION_HEADER_INDEX: usize = 9;

pub const MAX_EXTRA_DATA_BYTES: usize = 32;
pub const MAX_LOGS_BLOOM_SIZE: usize = 256;
pub const MAX_FEE_RECIPIENT_SIZE: usize = 20;

pub const MAX_BRANCH_PROOF_SIZE: usize = 20;

/// DomainType('0x07000000')
/// https://github.com/ethereum/consensus-specs/blob/dev/specs/altair/beacon-chain.md#domain-types
pub const DOMAIN_SYNC_COMMITTEE: [u8; 4] = [7, 0, 0, 0];

pub const PUBKEY_SIZE: usize = 48;
pub const SIGNATURE_SIZE: usize = 96;

const_assert!(SYNC_COMMITTEE_BITS_SIZE == SYNC_COMMITTEE_SIZE / 8);
