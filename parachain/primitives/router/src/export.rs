use core::slice::Iter;

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
		if dest != Here {
			log::trace!(target: "ethereum_blob_exporter", "skipped due to unmatched remote destination {dest:?}.");
			return Err(SendError::NotApplicable)
		}

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
			return Err(SendError::NotApplicable)
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

		let mut matcher = XcmConverter::new(&message, &bridged_network);
		let (payload, max_target_fee) = matcher.do_match().map_err(|err|{
			log::error!(target: "ethereum_blob_exporter", "unroutable due to pattern matching error '{err:?}'.");
			SendError::Unroutable
		})?;

		if max_target_fee.is_some() {
			log::error!(target: "ethereum_blob_exporter", "unroutable due not supporting max target fee.");
			return Err(SendError::Unroutable)
		}

		let (encoded, handler) = payload.abi_encode();

		let blob = BridgeMessage(para_id.into(), handler, encoded).encode();
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

#[derive(RuntimeDebug)]
enum NativeTokens {
	Unlock { asset: H160, destination: H160, amount: u128 },
}

#[derive(RuntimeDebug)]
enum OutboundPayload {
	NativeTokens(NativeTokens),
}

impl OutboundPayload {
	pub fn abi_encode(&self) -> (Vec<u8>, u16) {
		match self {
			Self::NativeTokens(NativeTokens::Unlock { asset, destination, amount }) => {
				let inner = ethabi::encode(&[Token::Tuple(vec![
					Token::Address(*asset),
					Token::Address(*destination),
					Token::Uint((*amount).into()),
				])]);
				let message = ethabi::encode(&[Token::Tuple(vec![
					Token::Uint(0.into()), // Unlock action
					Token::Bytes(inner),
				])]);

				(message, 1)
			},
		}
	}
}

/// Errors that can be thrown to the pattern matching step.
#[derive(RuntimeDebug)]
enum XcmConverterError {
	UnexpectedEndOfXcm,
	TargetFeeExpected,
	BuyExecutionExpected,
	EndOfXcmMessageExpected,
	ReserveAssetDepositedExpected,
	NoReserveAssets,
	FilterDoesNotConsumeAllReservedAssets,
	TooManyAssets,
	AssetsNotFound,
	AssetNotConcreteFungible,
	ZeroAssetTransfer,
	BeneficiaryResolutionFailed,
	AssetResolutionFailed,
}

struct XcmConverter<'a, Call> {
	iter: Iter<'a, Instruction<Call>>,
	bridged_location: &'a NetworkId,
}
impl<'a, Call> XcmConverter<'a, Call> {
	pub fn new(message: &'a Xcm<Call>, bridged_location: &'a NetworkId) -> Self {
		Self { iter: message.inner().iter(), bridged_location }
	}

	pub fn do_match(
		&mut self,
	) -> Result<(OutboundPayload, Option<&'a MultiAsset>), XcmConverterError> {
		use XcmConverterError::*;

		// Get target fees if specified.
		let max_target_fee = self.get_fee_info()?;

		// Get deposit reserved asset
		let result = self.get_reserve_deposited_asset()?;

		// All xcm instructions must be consumed before exit.
		if self.next().is_ok() {
			Err(EndOfXcmMessageExpected)
		} else {
			Ok((result, max_target_fee))
		}
	}

	fn get_fee_info(&mut self) -> Result<Option<&'a MultiAsset>, XcmConverterError> {
		use XcmConverterError::*;
		let execution_fee = match self.next()? {
			WithdrawAsset(fee_asset) => match self.next()? {
				BuyExecution { fees: execution_fee, weight_limit: Unlimited }
					if fee_asset.len() == 1 && fee_asset.contains(&execution_fee) =>
					Some(execution_fee),
				_ => return Err(BuyExecutionExpected),
			},
			UnpaidExecution { check_origin: None, weight_limit: Unlimited } => None,
			_ => return Err(TargetFeeExpected),
		};
		Ok(execution_fee)
	}

	fn get_reserve_deposited_asset(&mut self) -> Result<OutboundPayload, XcmConverterError> {
		use XcmConverterError::*;
		let (assets, beneficiary) = if let ReserveAssetDeposited(reserved_assets) = self.next()? {
			if reserved_assets.len() == 0 {
				return Err(NoReserveAssets)
			}
			if let (ClearOrigin, DepositAsset { assets, beneficiary }) =
				(self.next()?, self.next()?)
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

		// assert that the benificiary is ethereum account key 20
		let destination = {
			if let MultiLocation {
				parents: 0,
				interior: X1(AccountKey20 { network: Some(network), key }),
			} = beneficiary
			{
				if network != self.bridged_location {
					return Err(BeneficiaryResolutionFailed)
				}
				H160(*key)
			} else {
				return Err(BeneficiaryResolutionFailed)
			}
		};

		let (asset, amount) = {
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

			// extract ERC20 contract address
			if let MultiLocation {
				parents: 0,
				interior: X1(AccountKey20 { network: Some(network), key }),
			} = asset_location
			{
				if network != self.bridged_location {
					return Err(AssetResolutionFailed)
				}
				(H160(*key), *amount)
			} else {
				return Err(AssetResolutionFailed)
			}
		};

		Ok(OutboundPayload::NativeTokens(NativeTokens::Unlock { asset, destination, amount }))
	}

	fn next(&mut self) -> Result<&'a Instruction<Call>, XcmConverterError> {
		self.iter.next().ok_or(XcmConverterError::UnexpectedEndOfXcm)
	}
}

#[cfg(test)]
mod tests {
	use frame_support::parameter_types;
	use hex_literal::hex;

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
	}

	#[test]
	fn exporter_exports_valid_xcm() {
		let expected_ticket = hex!("e80300000100810300000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000600000000000000000000000001000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000003e8");
		let expected_ticket_hash =
			hex!("1454532f17679d9bfd775fef52de6c0598e34def65ef19ac06c11af013d6ca0f");

		let network = Ethereum { chain_id: 1 };
		let mut destination: Option<InteriorMultiLocation> = Here.into();

		let mut universal_source: Option<InteriorMultiLocation> =
			Some(X2(GlobalConsensus(Polkadot), Parachain(1000)));

		let token_address: [u8; 20] = hex!("1000000000000000000000000000000000000000");
		let beneficiary_address: [u8; 20] = hex!("2000000000000000000000000000000000000000");

		let channel: u32 = 0;
		let assets: MultiAssets = vec![MultiAsset {
			id: Concrete(X1(AccountKey20 { network: Some(network), key: token_address }).into()),
			fun: Fungible(1000),
		}]
		.into();
		let filter: MultiAssetFilter = assets.clone().into();

		let mut message: Option<Xcm<()>> = Some(
			vec![
				UnpaidExecution { weight_limit: Unlimited, check_origin: None },
				ReserveAssetDeposited(assets),
				ClearOrigin,
				DepositAsset {
					assets: filter,
					beneficiary: X1(AccountKey20 {
						network: Some(network),
						key: beneficiary_address,
					})
					.into(),
				},
			]
			.into(),
		);

		let result =
			ToBridgeEthereumBlobExporter::<RelayNetwork, BridgedNetwork, MockSubmitter>::validate(
				network,
				channel,
				&mut universal_source,
				&mut destination,
				&mut message,
			);

		assert_eq!(
			result,
			Ok(((expected_ticket.into(), expected_ticket_hash.into()), vec![].into()))
		);
	}
}
