#[subxt::subxt(
    runtime_metadata_path = "bridge-hub-metadata.bin",
    derive_for_all_types = "Clone",
    derive_for_type(path = "bridge_hub_polkadot_runtime::RuntimeCall", derive = "codec::Encode", recursive),
    derive_for_type(path = "sp_arithmetic::per_things::Perbill", derive = "codec::CompactAs"),
    derive_for_type(path = "polkadot_parachain_primitives::primitives::Id", derive = "codec::CompactAs"),
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
