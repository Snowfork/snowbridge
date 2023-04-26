use codec::{Decode, Encode};
use ethabi::{self, Token};
use frame_support::{ensure, log, traits::Get};
use snowbridge_core::{ParaId, SubmitMessage};
use sp_core::{RuntimeDebug, H160};
use sp_std::{marker::PhantomData, prelude::*};
use xcm::v3::prelude::*;
use xcm_executor::traits::ExportXcm;

#[derive(Encode, Decode)]
struct BridgeMessage(ParaId, u16, Vec<u8>);

pub struct ToBridgeEthereumBlobExporter<RelayNetwork, BridgedNetwork, Submitter>(
	PhantomData<(RelayNetwork, BridgedNetwork, Submitter)>,
);
impl<RelayNetwork: Get<NetworkId>, BridgedNetwork: Get<NetworkId>, Submitter: SubmitMessage>
	ExportXcm for ToBridgeEthereumBlobExporter<RelayNetwork, BridgedNetwork, Submitter>
{
	type Ticket = (Vec<u8>, XcmHash);

	fn validate(
		network: NetworkId,
		_channel: u32,
		universal_source: &mut Option<InteriorMultiLocation>,
		destination: &mut Option<InteriorMultiLocation>,
		message: &mut Option<Xcm<()>>,
	) -> SendResult<Self::Ticket> {
		let bridged_network = BridgedNetwork::get();
		if network != bridged_network {
			log::trace!(target: "ethereum_blob_exporter", "skipped due to unmatched bridge network {network:?}.");
			return Err(SendError::NotApplicable)
		}

		let dest = destination.take().ok_or(SendError::MissingArgument)?;
		if let Err((dest, _)) = dest.pushed_front_with(GlobalConsensus(bridged_network)) {
			*destination = Some(dest);
			log::trace!(target: "ethereum_blob_exporter", "skipped due to invalid destination '{dest:?}'.");
			return Err(SendError::NotApplicable)
		};

		let (local_net, local_sub) = universal_source
			.take()
			.ok_or_else(|| {
				log::error!(target: "ethereum_blob_exporter", "universal source not provided.");
				SendError::MissingArgument
			})?
			.split_global()
			.map_err(|()| {
				log::error!(target: "ethereum_blob_exporter", "could not get global consensus from universal source '{universal_source:?}'.");
				SendError::Unroutable
			})?;

		if local_net != RelayNetwork::get() {
			log::trace!(target: "ethereum_blob_exporter", "skipped due to unmatched relay network {local_net:?}.");
			return Err(SendError::NotApplicable);
		}

		let para_id = match local_sub {
			X1(Parachain(para_id)) => para_id,
			_ => {
				log::error!(target: "ethereum_blob_exporter", "could not get parachain id from universal source '{local_sub:?}'.");
				return Err(SendError::MissingArgument)
			},
		};

		let message = message.take().ok_or_else(|| {
			log::error!(target: "ethereum_blob_exporter", "xcm message not provided.");
			SendError::MissingArgument
		})?;

		let parse_info = match_xcm_pattern(&message).map_err(|err| {
			log::error!(target: "ethereum_blob_exporter", "unroutable due to pattern matching error '{err:?}'.");
			SendError::Unroutable
		})?;

		let (encoded_payload, handler) =
			validate_and_encode(&local_net, &bridged_network, &parse_info).map_err(|err| {
				log::error!(target: "ethereum_blob_exporter", "unroutable due to validation error '{err:?}'.");
				SendError::Unroutable
			})?;

		let blob = BridgeMessage(para_id.into(), handler, encoded_payload).encode();
		let hash: [u8; 32] = sp_io::hashing::blake2_256(blob.as_slice());

		log::info!(target: "ethereum_blob_exporter", "message validated {hash:#?}.");

		// TODO: Fees if any currently returning empty multi assets as cost
		Ok(((blob, hash), MultiAssets::default()))
	}

	fn deliver((blob, hash): (Vec<u8>, XcmHash)) -> Result<XcmHash, SendError> {
		let mut blob = blob.clone();
		let mut input: &[u8] = blob.as_mut();
		let BridgeMessage(source_id, handler, payload) = BridgeMessage::decode(&mut input)
			.map_err(|err| {
				log::error!(target: "ethereum_blob_exporter", "undeliverable due to decoding error '{err:?}'.");
				SendError::NotApplicable
			})?;
		Submitter::submit(&source_id, handler, payload.as_ref()).map_err(|err| {
			log::error!(target: "ethereum_blob_exporter", "undeliverable due to submitter error '{err:?}'.");
			SendError::Unroutable
		})?;
		log::info!(target: "ethereum_blob_exporter", "message delivered {hash:#?}.");
		Ok(hash)
	}
}

/// Represents a type of XCM message that was pattern matched. e.g. Asset transfers, Asset create,
/// Transact. At this level of abstraction we have picked out the pieces of the xcm we care about.
#[derive(RuntimeDebug)]
enum XcmMessagePattern {
	AssetTransfer {
		assets: MultiAssets,
		beneficiary: MultiLocation,
		max_target_fee: Option<MultiAsset>,
	},
}

/// Errors that can be thrown to the pattern matching step.
#[derive(RuntimeDebug)]
enum XcmPatternMatchError {
	UnexpectedEndOfXcm,
	TargetFeeExpected,
	BuyExecutionExpected,
	EndOfXcmMessageExpected,
	ReserveAssetDepositedExpected,
	NoReserveAssets,
	FilterDoesNotConsumeAllReservedAssets,
}

/// Figures out what this xcm message looks like. Does basic validation to make sure the
/// xcm message is sound.
fn match_xcm_pattern(message: &Xcm<()>) -> Result<XcmMessagePattern, XcmPatternMatchError> {
	use XcmPatternMatchError::*;

	let mut next_instruction = {
		let mut it = message.iter();
		move || it.next().ok_or(UnexpectedEndOfXcm)
	};

	// Get target fees if specified.
	let max_target_fee = match next_instruction()? {
		WithdrawAsset(fee_asset) => match next_instruction()? {
			BuyExecution { fees: execution_fee, weight_limit: Unlimited }
				if fee_asset.len() == 1 && fee_asset.contains(execution_fee) =>
				Some(execution_fee),
			_ => return Err(BuyExecutionExpected),
		},
		UnpaidExecution { check_origin: None, weight_limit: Unlimited } => None,
		_ => return Err(TargetFeeExpected),
	};

	// Get deposit reserved asset
	let (assets, beneficiary) = if let ReserveAssetDeposited(reserved_assets) = next_instruction()?
	{
		if reserved_assets.len() == 0 {
			return Err(NoReserveAssets)
		}
		if let (ClearOrigin, DepositAsset { assets, beneficiary }) =
			(next_instruction()?, next_instruction()?)
		{
			if reserved_assets.inner().iter().any(|asset| !assets.matches(asset)) {
				return Err(FilterDoesNotConsumeAllReservedAssets)
			}
			(reserved_assets, beneficiary)
		} else {
			return Err(ReserveAssetDepositedExpected)
		}
	} else {
		return Err(ReserveAssetDepositedExpected)
	};

	// All xcm instructions must be consumed before exit.
	if next_instruction().is_ok() {
		Err(EndOfXcmMessageExpected)
	} else {
		Ok(XcmMessagePattern::AssetTransfer {
			assets: assets.clone(),
			beneficiary: beneficiary.clone(),
			max_target_fee: max_target_fee.map(|fee| fee.clone()),
		})
	}
}

/// Errors that can occur during validation of the xcm pattern.
#[derive(RuntimeDebug)]
enum ValidationError {
	TargetFeesNotSupported,
	TooManyAssets,
	AssetsNotFound,
	AssetNotConcreteFungible,
	ZeroAssetTransfer,
	EthABIEncodeError,
	BeneficiaryResolutionFailed,
	AssetResolutionFailed,
}

/// Validates the Xcm pattern to ensure that all assets and locations are
/// correct and then produces the EthABI encoded message equivalent and
/// relevant handler id.
/// e.g. The XcmMessagePattern::AssetTransfer pattern must be routed to
/// NativeTokens handler id and create and Unlock message.
/// e.g. The XcmMessagePattern::AssetTransfer with substrate native asset would
/// need route to the handler for substrate tokens and create a Mint message.
fn validate_and_encode(
	universal_location: &NetworkId,
	bridged_location: &NetworkId,
	message_type: &XcmMessagePattern,
) -> Result<(Vec<u8>, u16), ValidationError> {
	use ValidationError::*;
	const NATIVE_TOKENS_HANDLER: u16 = 1;
	const UNLOCK_ACTION: u16 = 0;
	match message_type {
		XcmMessagePattern::AssetTransfer { assets, beneficiary, max_target_fee } => {
			// We do not support target fees
			ensure!(max_target_fee.is_none(), TargetFeesNotSupported);

			let (asset_address, amount) = {
				// We only support a single asset at a time.
				ensure!(assets.len() == 1, TooManyAssets);

				// Ensure asset is concrete and fungible.
				let asset = assets.get(0).ok_or(AssetsNotFound)?;
				let (asset_location, amount) =
					if let MultiAsset { id: Concrete(location), fun: Fungible(amount) } = asset {
						(location, amount)
					} else {
						return Err(AssetNotConcreteFungible)
					};

				ensure!(*amount > 0, ZeroAssetTransfer);

				let (network, location) = ensure_is_remote(*universal_location, *asset_location)
					.ok()
					.ok_or(AssetResolutionFailed)?;
				ensure!(&network == bridged_location, AssetResolutionFailed);
				(
					location_to_eth_address(*bridged_location, location)
						.ok_or(AssetResolutionFailed)?,
					amount,
				)
			};

			// Ensure benificiary is Ethereum address.
			let destination_address = {
				let (network, location) = ensure_is_remote(*universal_location, *beneficiary)
					.ok()
					.ok_or(BeneficiaryResolutionFailed)?;
				ensure!(&network == bridged_location, BeneficiaryResolutionFailed);
				location_to_eth_address(*bridged_location, location)
					.ok_or(BeneficiaryResolutionFailed)?
			};

			let inner = Token::Tuple(vec![
				Token::Address(asset_address),
				Token::Address(destination_address),
				Token::Uint((*amount).into()),
			])
			.to_bytes()
			.ok_or(EthABIEncodeError)?;

			let message =
				Token::Tuple(vec![Token::Uint(UNLOCK_ACTION.into()), Token::Bytes(inner)]);

			Ok((message.to_bytes().ok_or(EthABIEncodeError)?, NATIVE_TOKENS_HANDLER))
		},
	}
}

// copied from ~/polkadot/xcm/xcm-builder/src/universal_exports.rs
// TODO: Merge latest from upstream where this function is exported as public.
fn ensure_is_remote(
	universal_local: impl Into<InteriorMultiLocation>,
	dest: impl Into<MultiLocation>,
) -> Result<(NetworkId, InteriorMultiLocation), MultiLocation> {
	let dest = dest.into();
	let universal_local = universal_local.into();
	let local_net = match universal_local.global_consensus() {
		Ok(x) => x,
		Err(_) => return Err(dest),
	};
	let universal_destination: InteriorMultiLocation = universal_local
		.into_location()
		.appended_with(dest)
		.map_err(|x| x.1)?
		.try_into()?;
	let (remote_dest, remote_net) = match universal_destination.split_first() {
		(d, Some(GlobalConsensus(n))) if n != local_net => (d, n),
		_ => return Err(dest),
	};
	Ok((remote_net, remote_dest))
}

fn location_to_eth_address(
	eth_network: NetworkId,
	location: InteriorMultiLocation,
) -> Option<H160> {
	if let X1(AccountKey20 { network, key }) = location {
		if network == None || network == Some(eth_network) {
			Some(H160(key))
		} else {
			None
		}
	} else {
		None
	}
}

#[cfg(test)]
mod tests {
	use frame_support::parameter_types;

	use super::*;

	parameter_types! {
		pub const RelayNetwork: NetworkId = Polkadot;
		pub const BridgedNetwork: NetworkId =  Ethereum{ chain_id: 1 };
	}

	struct MockSubmitter;
	impl SubmitMessage for MockSubmitter {
		fn submit(
			_source_id: &snowbridge_core::ParaId,
			_handler: u16,
			_payload: &[u8],
		) -> sp_runtime::DispatchResult {
			Ok(())
		}
	}

	#[test]
	fn exporter_with_unknown_network_yields_not_applicable() {
		let network = Ethereum { chain_id: 1337 };
		let channel: u32 = 0;
		let mut universal_source: Option<InteriorMultiLocation> = None;
		let mut destination: Option<InteriorMultiLocation> = None;
		let mut message: Option<Xcm<()>> = None;

		let result =
			ToBridgeEthereumBlobExporter::<RelayNetwork, BridgedNetwork, MockSubmitter>::validate(
				network,
				channel,
				&mut universal_source,
				&mut destination,
				&mut message,
			);
		assert_eq!(result, Err(SendError::NotApplicable));
	}

	#[test]
	fn exporter_with_invalid_destination_yields_missing_argument() {
		let network = Ethereum { chain_id: 1 };
		let channel: u32 = 0;
		let mut universal_source: Option<InteriorMultiLocation> = None;
		let mut destination: Option<InteriorMultiLocation> = None;
		let mut message: Option<Xcm<()>> = None;

		let result =
			ToBridgeEthereumBlobExporter::<RelayNetwork, BridgedNetwork, MockSubmitter>::validate(
				network,
				channel,
				&mut universal_source,
				&mut destination,
				&mut message,
			);
		assert_eq!(result, Err(SendError::MissingArgument));
	}

	#[test]
	fn exporter_with_x8_destination_yields_not_applicable() {
		let network = Ethereum { chain_id: 1 };
		let channel: u32 = 0;
		let mut universal_source: Option<InteriorMultiLocation> = None;
		let mut destination: Option<InteriorMultiLocation> = Some(X8(
			OnlyChild, OnlyChild, OnlyChild, OnlyChild, OnlyChild, OnlyChild, OnlyChild, OnlyChild,
		));
		let mut message: Option<Xcm<()>> = None;

		let expected_destination = destination.clone();
		let result =
			ToBridgeEthereumBlobExporter::<RelayNetwork, BridgedNetwork, MockSubmitter>::validate(
				network,
				channel,
				&mut universal_source,
				&mut destination,
				&mut message,
			);
		assert_eq!(result, Err(SendError::NotApplicable));
		assert_eq!(destination, expected_destination);
	}

	#[test]
	fn exporter_without_universal_source_yields_missing_argument() {
		let network = Ethereum { chain_id: 1 };
		let channel: u32 = 0;
		let mut universal_source: Option<InteriorMultiLocation> = None;
		let mut destination: Option<InteriorMultiLocation> = Here.into();
		let mut message: Option<Xcm<()>> = None;

		let result =
			ToBridgeEthereumBlobExporter::<RelayNetwork, BridgedNetwork, MockSubmitter>::validate(
				network,
				channel,
				&mut universal_source,
				&mut destination,
				&mut message,
			);
		assert_eq!(result, Err(SendError::MissingArgument));
		assert_eq!(destination, None);
	}

	#[test]
	fn exporter_without_global_universal_location_yields_unroutable() {
		let network = Ethereum { chain_id: 1 };
		let channel: u32 = 0;
		let mut universal_source: Option<InteriorMultiLocation> = Here.into();
		let mut destination: Option<InteriorMultiLocation> = Here.into();
		let mut message: Option<Xcm<()>> = None;

		let result =
			ToBridgeEthereumBlobExporter::<RelayNetwork, BridgedNetwork, MockSubmitter>::validate(
				network,
				channel,
				&mut universal_source,
				&mut destination,
				&mut message,
			);
		assert_eq!(result, Err(SendError::Unroutable));
		assert_eq!(destination, None);
	}

	#[test]
	fn exporter_with_remote_universal_source_yields_not_applicable() {
		let network = Ethereum { chain_id: 1 };
		let channel: u32 = 0;
		let mut universal_source: Option<InteriorMultiLocation> =
			Some(X2(GlobalConsensus(Kusama), Parachain(1000)));
		let mut destination: Option<InteriorMultiLocation> = Here.into();
		let mut message: Option<Xcm<()>> = None;

		let result =
			ToBridgeEthereumBlobExporter::<RelayNetwork, BridgedNetwork, MockSubmitter>::validate(
				network,
				channel,
				&mut universal_source,
				&mut destination,
				&mut message,
			);
		assert_eq!(result, Err(SendError::NotApplicable));
		assert_eq!(destination, None);
	}

	#[test]
	fn exporter_without_para_id_in_source_yields_missing_argument() {
		let network = Ethereum { chain_id: 1 };
		let channel: u32 = 0;
		let mut universal_source: Option<InteriorMultiLocation> =
			Some(X1(GlobalConsensus(Polkadot)));
		let mut destination: Option<InteriorMultiLocation> = Here.into();
		let mut message: Option<Xcm<()>> = None;

		let result =
			ToBridgeEthereumBlobExporter::<RelayNetwork, BridgedNetwork, MockSubmitter>::validate(
				network,
				channel,
				&mut universal_source,
				&mut destination,
				&mut message,
			);
		assert_eq!(result, Err(SendError::MissingArgument));
		assert_eq!(destination, None);
	}

	#[test]
	fn exporter_complex_para_id_in_source_yields_missing_argument() {
		let network = Ethereum { chain_id: 1 };
		let channel: u32 = 0;
		let mut universal_source: Option<InteriorMultiLocation> =
			Some(X3(GlobalConsensus(Polkadot), Parachain(1000), PalletInstance(12)));
		let mut destination: Option<InteriorMultiLocation> = Here.into();
		let mut message: Option<Xcm<()>> = None;

		let result =
			ToBridgeEthereumBlobExporter::<RelayNetwork, BridgedNetwork, MockSubmitter>::validate(
				network,
				channel,
				&mut universal_source,
				&mut destination,
				&mut message,
			);
		assert_eq!(result, Err(SendError::MissingArgument));
		assert_eq!(destination, None);
	}

	#[test]
	fn exporter_without_xcm_message_yields_missing_argument() {
		let network = Ethereum { chain_id: 1 };
		let channel: u32 = 0;
		let mut universal_source: Option<InteriorMultiLocation> =
			Some(X2(GlobalConsensus(Polkadot), Parachain(1000)));
		let mut destination: Option<InteriorMultiLocation> = Here.into();
		let mut message: Option<Xcm<()>> = None;

		let result =
			ToBridgeEthereumBlobExporter::<RelayNetwork, BridgedNetwork, MockSubmitter>::validate(
				network,
				channel,
				&mut universal_source,
				&mut destination,
				&mut message,
			);
		assert_eq!(result, Err(SendError::MissingArgument));
		assert_eq!(destination, None);
	}

	#[test]
	fn exporter_test() {
		let network = Ethereum { chain_id: 1 };
		let mut destination: Option<InteriorMultiLocation> = Here.into();

		let mut universal_source: Option<InteriorMultiLocation> =
			Some(X2(GlobalConsensus(Polkadot), Parachain(1000)));

		let channel: u32 = 0;
		let mut message: Option<Xcm<()>> = None;

		let result =
			ToBridgeEthereumBlobExporter::<RelayNetwork, BridgedNetwork, MockSubmitter>::validate(
				network,
				channel,
				&mut universal_source,
				&mut destination,
				&mut message,
			);
		assert_eq!(result, Err(SendError::ExceedsMaxMessageSize));
		assert_eq!(destination, None);
	}
}
