use asset_hub_westend_runtime::runtime_types::xcm::{VersionedLocation as AssetHubVersionedLocation, VersionedXcm as AssetHubVersionedXcm};
use penpal_westend_runtime::runtime_types::xcm::{VersionedLocation as PenpalVersionedLocation, VersionedXcm as PenpalVersionedXcm};
use penpal_westend_runtime::runtime_types::staging_xcm::v5::{location::Location as PenpalLocation, traits::Outcome};
use penpal_westend_runtime::runtime_types::staging_xcm::v5::junctions::Junctions::X1;

use crate::estimator::{Clients, DryRunResult, EstimatorError};
use codec::{Encode, Decode};

const ASSET_HUB_PARA_ID: u32 = 1000;

pub async fn dry_run_xcm(clients: &Clients, xcm: AssetHubVersionedXcm) -> Result<DryRunResult, EstimatorError> {
    println!("=== DESTINATION XCM ABOUT TO BE DRY RUN ON PENPAL ===");
    println!("Original AssetHub XCM: {:?}", xcm);
    
    let penpal_xcm = convert_xcm_to_penpal_format(&xcm)?;
    println!("Converted Penpal XCM: {:?}", penpal_xcm);
    println!("=== END DESTINATION XCM ===");

    let origin_location = PenpalVersionedLocation::V5(
        PenpalLocation {
            parents: 1,
            interior: X1([
                penpal_westend_runtime::runtime_types::staging_xcm::v5::junction::Junction::Parachain(ASSET_HUB_PARA_ID)
            ]),
        }
    );

    let runtime_api_call = penpal_westend_runtime::runtime::apis()
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
        .map_err(|e| EstimatorError::InvalidCommand(format!("Failed to dry run XCM on Penpal: {:?}", e)))?;

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

            println!("Penpal dry run effects: {:?}", effects);

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

fn convert_xcm_to_penpal_format(xcm: &AssetHubVersionedXcm) -> Result<PenpalVersionedXcm, EstimatorError> {
    let encoded = xcm.encode();
    PenpalVersionedXcm::decode(&mut &encoded[..])
        .map_err(|e| EstimatorError::InvalidCommand(format!("Failed to convert XCM to Penpal format: {:?}", e)))
}

fn convert_xcm_from_penpal_format(forwarded: &(PenpalVersionedLocation, Vec<PenpalVersionedXcm>)) -> Result<(AssetHubVersionedLocation, Vec<AssetHubVersionedXcm>), EstimatorError> {
    let (penpal_location, penpal_xcms) = forwarded;

    let encoded_location = penpal_location.encode();
    let asset_hub_location = AssetHubVersionedLocation::decode(&mut &encoded_location[..])
        .map_err(|e| EstimatorError::InvalidCommand(format!("Failed to convert location from Penpal format: {:?}", e)))?;

    let mut asset_hub_xcms = Vec::new();
    for penpal_xcm in penpal_xcms {
        let encoded_xcm = penpal_xcm.encode();
        let asset_hub_xcm = AssetHubVersionedXcm::decode(&mut &encoded_xcm[..])
            .map_err(|e| EstimatorError::InvalidCommand(format!("Failed to convert XCM from Penpal format: {:?}", e)))?;
        asset_hub_xcms.push(asset_hub_xcm);
    }

    Ok((asset_hub_location, asset_hub_xcms))
}
