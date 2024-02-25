use subxt::subxt;

#[cfg_attr(
    feature = "rococo",
    subxt(
        runtime_metadata_path = "polkadot-metadata.bin",
        derive_for_all_types = "Clone",
    )
)]
#[cfg_attr(
    feature = "kusama",
    subxt(
        runtime_metadata_path = "polkadot-metadata.bin",
        derive_for_all_types = "Clone",
    )
)]
#[cfg_attr(
    feature = "polkadot",
    subxt(
        runtime_metadata_path = "polkadot-metadata.bin",
        derive_for_all_types = "Clone",
    )
)]
pub mod relay {}

#[cfg_attr(
    feature = "rococo",
    subxt(
        runtime_metadata_path = "bridge-hub-metadata.bin",
        derive_for_all_types = "Clone",
    )
)]
#[cfg_attr(
    feature = "kusama",
    subxt(
        runtime_metadata_path = "bridge-hub-metadata.bin",
        derive_for_all_types = "Clone",
    )
)]
#[cfg_attr(
    feature = "polkadot",
    subxt(
        runtime_metadata_path = "bridge-hub-metadata.bin",
        derive_for_all_types = "Clone",
    )
)]
pub mod bridge_hub {}
