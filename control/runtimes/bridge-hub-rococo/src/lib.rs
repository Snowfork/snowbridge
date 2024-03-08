#[subxt::subxt(
    runtime_metadata_path = "bridge-hub-metadata.bin",
    derive_for_all_types = "Clone",
)]
mod runtime {}

pub use runtime::*;
