pub const SLOTS_PER_EPOCH: u64 = 8;
pub const EPOCHS_PER_SYNC_COMMITTEE_PERIOD: u64 = 8;
pub const SYNC_COMMITTEE_SIZE: usize = 32;
pub const IS_MINIMAL: bool = true;

pub const GENESIS_FORK_VERSION: [u8; 4] =[0, 0, 0, 1]; // 0x00001020

pub const ALTAIR_FORK_VERSION: [u8; 4] =[1, 0, 0, 1]; // 0x01001020
pub const ALTAIR_FORK_EPOCH: u64 = 0;

pub const BELLATRIX_FORK_VERSION: [u8; 4] =[2, 0, 0, 1]; // 0x02001020
pub const BELLATRIX_FORK_EPOCH: u64 = 0;
