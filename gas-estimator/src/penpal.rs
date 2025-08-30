#[cfg(feature = "local")]
use asset_hub_westend_local_runtime::runtime_types::xcm::{
    VersionedLocation as AssetHubVersionedLocation, VersionedXcm as AssetHubVersionedXcm,
};
#[cfg(feature = "local")]
use penpal_westend_local_runtime::runtime_types::{
    sp_weights::weight_v2::Weight,
    staging_xcm::v5::{
        asset::AssetId,
        junction::Junction::Parachain,
        junctions::Junctions::{Here, X1},
        location::Location as PenpalLocation,
        traits::Outcome,
    },
    xcm::{
        VersionedAssetId, VersionedLocation as PenpalVersionedLocation,
        VersionedXcm as PenpalVersionedXcm,
    },
};

use crate::estimator::{Clients, DryRunResult, EstimatorError};
use codec::{Decode, Encode};

#[cfg(feature = "local")]
const ASSET_HUB_PARA_ID: u32 = 1000;

#[cfg(feature = "local")]
pub async fn dry_run_xcm(
    clients: &Clients,
    xcm: AssetHubVersionedXcm,
) -> Result<DryRunResult, EstimatorError> {
    let penpal_xcm = convert_xcm_to_penpal_format(&xcm)?;

    let origin_location = PenpalVersionedLocation::V5(PenpalLocation {
        parents: 1,
        interior: X1([Parachain(ASSET_HUB_PARA_ID)]),
    });

    let runtime_api_call = penpal_westend_local_runtime::runtime::apis()
        .dry_run_api()
        .dry_run_xcm(origin_location, penpal_xcm);

    let dry_run_result = clients
        .penpal_client
        .runtime_api()
        .at_latest()
        .await
        .map_err(|e| {
            EstimatorError::InvalidCommand(format!("Failed to get latest block: {:?}", e))
        })?
        .call(runtime_api_call)
        .await
        .map_err(|e| {
            EstimatorError::InvalidCommand(format!("Failed to dry run XCM on Penpal: {:?}", e))
        })?;

    match dry_run_result {
        Ok(effects) => {
            let success = matches!(effects.execution_result, Outcome::Complete { .. });

            let error_message = if success {
                None
            } else {
                Some(format!(
                    "XCM execution failed on Penpal: {:?}",
                    effects.execution_result
                ))
            };

            let forwarded_xcms = effects.forwarded_xcms;

            let forwarded_xcm = if forwarded_xcms.is_empty() {
                None
            } else {
                Some(convert_xcm_from_penpal_format(&forwarded_xcms[0])?)
            };

            Ok(DryRunResult {
                success,
                error_message,
                forwarded_xcm,
            })
        }
        Err(e) => Ok(DryRunResult {
            success: false,
            error_message: Some(format!("Penpal dry run API error: {:?}", e)),
            forwarded_xcm: None,
        }),
    }
}

#[cfg(feature = "local")]
fn convert_xcm_to_penpal_format(
    xcm: &AssetHubVersionedXcm,
) -> Result<PenpalVersionedXcm, EstimatorError> {
    let encoded = xcm.encode();
    PenpalVersionedXcm::decode(&mut &encoded[..]).map_err(|e| {
        EstimatorError::InvalidCommand(format!("Failed to convert XCM to Penpal format: {:?}", e))
    })
}

#[cfg(feature = "local")]
fn convert_xcm_from_penpal_format(
    forwarded: &(PenpalVersionedLocation, Vec<PenpalVersionedXcm>),
) -> Result<(AssetHubVersionedLocation, Vec<AssetHubVersionedXcm>), EstimatorError> {
    let (penpal_location, penpal_xcms) = forwarded;

    let encoded_location = penpal_location.encode();
    let asset_hub_location = AssetHubVersionedLocation::decode(&mut &encoded_location[..])
        .map_err(|e| {
            EstimatorError::InvalidCommand(format!(
                "Failed to convert location from Penpal format: {:?}",
                e
            ))
        })?;

    let mut asset_hub_xcms = Vec::new();
    for penpal_xcm in penpal_xcms {
        let encoded_xcm = penpal_xcm.encode();
        let asset_hub_xcm = AssetHubVersionedXcm::decode(&mut &encoded_xcm[..]).map_err(|e| {
            EstimatorError::InvalidCommand(format!(
                "Failed to convert XCM from Penpal format: {:?}",
                e
            ))
        })?;
        asset_hub_xcms.push(asset_hub_xcm);
    }

    Ok((asset_hub_location, asset_hub_xcms))
}

#[cfg(feature = "local")]
async fn query_penpal_xcm_weight(
    clients: &Clients,
    xcm: &PenpalVersionedXcm,
) -> Result<Weight, EstimatorError> {
    let runtime_api_call = penpal_westend_local_runtime::runtime::apis()
        .xcm_payment_api()
        .query_xcm_weight(xcm.clone());

    let weight_result = clients
        .penpal_client
        .runtime_api()
        .at_latest()
        .await
        .map_err(|e| {
            EstimatorError::InvalidCommand(format!("Failed to get latest block: {:?}", e))
        })?
        .call(runtime_api_call)
        .await
        .map_err(|e| {
            EstimatorError::InvalidCommand(format!("Failed to query XCM weight on Penpal: {:?}", e))
        })?;

    weight_result.map_err(|e| {
        EstimatorError::InvalidCommand(format!("Penpal XCM weight query error: {:?}", e))
    })
}

#[cfg(feature = "local")]
async fn query_penpal_weight_to_asset_fee(
    clients: &Clients,
    weight: &Weight,
) -> Result<u128, EstimatorError> {
    let dot_asset_location = PenpalLocation {
        parents: 1,
        interior: Here,
    };
    let dot_asset = VersionedAssetId::V5(AssetId(dot_asset_location));

    let runtime_api_call = penpal_westend_local_runtime::runtime::apis()
        .xcm_payment_api()
        .query_weight_to_asset_fee(weight.clone(), dot_asset);

    let fee_result = clients
        .penpal_client
        .runtime_api()
        .at_latest()
        .await
        .map_err(|e| {
            EstimatorError::InvalidCommand(format!("Failed to get latest block: {:?}", e))
        })?
        .call(runtime_api_call)
        .await
        .map_err(|e| {
            EstimatorError::InvalidCommand(format!(
                "Failed to query weight to asset fee on Penpal: {:?}",
                e
            ))
        })?;

    fee_result.map_err(|e| {
        EstimatorError::InvalidCommand(format!("Penpal weight to asset fee query error: {:?}", e))
    })
}

#[cfg(feature = "local")]
pub async fn calculate_execution_fee(
    clients: &Clients,
    xcm: &AssetHubVersionedXcm,
) -> Result<u128, EstimatorError> {
    let penpal_xcm = convert_xcm_to_penpal_format(xcm)?;
    let weight = query_penpal_xcm_weight(clients, &penpal_xcm).await?;
    let fee_in_dot = query_penpal_weight_to_asset_fee(clients, &weight).await?;

    Ok(fee_in_dot)
}
