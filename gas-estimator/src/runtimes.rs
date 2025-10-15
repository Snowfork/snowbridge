#[cfg(feature = "local")]
pub use asset_hub_westend_local_runtime::{
    runtime as asset_hub_runtime,
    runtime_types::{
        bounded_collections::bounded_vec::BoundedVec,
        staging_xcm::v5::{
            asset::{
                Asset, AssetFilter, AssetId, Assets, Fungibility, WildAsset,
            },
            junction::{Junction, NetworkId},
            junctions::Junctions,
            location::Location,
            Hint, Instruction, Xcm,
        },
        xcm::{double_encoded::DoubleEncoded, v3::OriginKind, VersionedXcm},
    },
};

#[cfg(feature = "westend")]
pub use asset_hub_westend_runtime::{
    runtime as asset_hub_runtime,
    runtime_types::{
        bounded_collections::bounded_vec::BoundedVec,
        staging_xcm::v5::{
            asset::{
                Asset, AssetFilter, AssetId, Assets, Fungibility, WildAsset,
            },
            junction::{Junction, NetworkId},
            junctions::Junctions,
            location::Location,
            Hint, Instruction, Xcm,
        },
        xcm::{double_encoded::DoubleEncoded, v3::OriginKind, VersionedXcm},
    },
};

#[cfg(feature = "paseo")]
pub use asset_hub_paseo_runtime::{
    runtime as asset_hub_runtime,
    runtime_types::{
        bounded_collections::bounded_vec::BoundedVec,
        staging_xcm::v5::{
            asset::{
                Asset, AssetFilter, AssetId, Assets, Fungibility, WildAsset,
            },
            junction::{Junction, NetworkId},
            junctions::Junctions,
            location::Location,
            Hint, Instruction, Xcm,
        },
        xcm::{double_encoded::DoubleEncoded, v3::OriginKind, VersionedXcm},
    },
};

// Bridge Hub runtime imports
#[cfg(feature = "local")]
pub use bridge_hub_westend_local_runtime::{
    runtime as bridge_hub_runtime,
    runtime_types::{
        snowbridge_verification_primitives::{EventProof, Log, Proof},
        staging_xcm::v5::{
            asset::Fungibility as BridgeHubFungibility,
            junction::{Junction as BridgeHubJunction, Junction::Parachain as BridgeHubParachain},
            junctions::Junctions as BridgeHubJunctions,
            location::Location as BridgeHubLocation,
        },
        xcm::{
            VersionedAssets as BridgeHubVersionedAssets,
            VersionedLocation as BridgeHubVersionedLocation,
            VersionedXcm as BridgeHubVersionedXcm,
        },
    },
};

#[cfg(feature = "westend")]
pub use bridge_hub_westend_runtime::{
    runtime as bridge_hub_runtime,
    runtime_types::{
        snowbridge_verification_primitives::{EventProof, Log, Proof},
        staging_xcm::v5::{
            asset::Fungibility as BridgeHubFungibility,
            junction::{Junction as BridgeHubJunction, Junction::Parachain as BridgeHubParachain},
            junctions::Junctions as BridgeHubJunctions,
            location::Location as BridgeHubLocation,
        },
        xcm::{
            VersionedAssets as BridgeHubVersionedAssets,
            VersionedLocation as BridgeHubVersionedLocation,
            VersionedXcm as BridgeHubVersionedXcm,
        },
    },
};

#[cfg(feature = "paseo")]
pub use bridge_hub_paseo_runtime::{
    runtime as bridge_hub_runtime,
    runtime_types::{
        snowbridge_verification_primitives::{EventProof, Log, Proof},
        staging_xcm::v5::{
            asset::Fungibility as BridgeHubFungibility,
            junction::{Junction as BridgeHubJunction, Junction::Parachain as BridgeHubParachain},
            junctions::Junctions as BridgeHubJunctions,
            location::Location as BridgeHubLocation,
        },
        xcm::{
            VersionedAssets as BridgeHubVersionedAssets,
            VersionedLocation as BridgeHubVersionedLocation,
            VersionedXcm as BridgeHubVersionedXcm,
        },
    },
};
