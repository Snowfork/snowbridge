pub const SLOTS_PER_EPOCH: u64 = 32;
pub const EPOCHS_PER_SYNC_COMMITTEE_PERIOD: u64 = 256;
pub const SYNC_COMMITTEE_SIZE: usize = 512;
pub const IS_MINIMAL: bool = false;

pub const GENESIS_FORK_VERSION: [u8; 4] =[0, 0, 16, 32]; // 0x00001020

pub const ALTAIR_FORK_VERSION: [u8; 4] =[1, 0, 16, 32]; // 0x01001020
pub const ALTAIR_FORK_EPOCH: u64 = 36660;

pub const BELLATRIX_FORK_VERSION: [u8; 4] =[2, 0, 16, 32]; // 0x02001020
pub const BELLATRIX_FORK_EPOCH: u64 = 112260;
