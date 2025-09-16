#[subxt::subxt(
    runtime_metadata_path = "asset-hub-metadata.bin",
    derive_for_all_types = "Clone, WrapperTypeEncode",
    derive_for_type(path = "asset_hub_kusama_runtime::RuntimeCall", derive = "codec::Encode"),
)]
mod runtime {}

pub use runtime::*;
