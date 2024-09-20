#[subxt::subxt(
    runtime_metadata_path = "asset-hub-metadata.bin",
    derive_for_all_types = "Clone"
)]
mod runtime {}

pub use runtime::*;
