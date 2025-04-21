#[subxt::subxt(
    runtime_metadata_path = "polkadot-metadata.bin",
    derive_for_all_types = "Clone"
)]
mod runtime {}

pub use runtime::*;
