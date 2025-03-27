#[subxt::subxt(
    runtime_metadata_path = "bridge-hub-metadata.bin",
    derive_for_all_types = "Clone",
    substitute_type(
        path = "snowbridge_beacon_primitives::updates::CheckpointUpdate",
        with = "::subxt::utils::Static<::snowbridge_beacon_primitives::updates::CheckpointUpdate<512>>",
    ),
    substitute_type(
        path = "sp_arithmetic::fixed_point::FixedU128",
        with = "::subxt::utils::Static<::sp_arithmetic::fixed_point::FixedU128>",
    )
)]
mod runtime {}

pub use runtime::*;

pub const CHAIN_ID: u64 = 1;
