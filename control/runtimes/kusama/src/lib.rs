#[subxt::subxt(
    runtime_metadata_path = "polkadot-metadata.bin",
    derive_for_all_types = "Clone",
    derive_for_type(path = "staging_kusama_runtime::RuntimeCall", derive = "codec::Encode", recursive),
    derive_for_type(path = "sp_arithmetic::per_things::Perbill", derive = "codec::CompactAs"),
    derive_for_type(path = "sp_arithmetic::per_things::PerU16", derive = "codec::CompactAs"),
    derive_for_type(path = "polkadot_parachain_primitives::primitives::Id", derive = "codec::CompactAs")
)]
mod runtime {}

pub use runtime::*;
