// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
//! Converts XCM messages into simpler commands that can be processed by the Gateway contract
use core::slice::Iter;

use codec::{Decode, Encode};

use frame_support::{ensure, traits::Get};
use snowbridge_core::outbound::{AgentExecuteCommand, Command, Message, SendMessage};
use sp_core::{H160, H256};
use sp_std::{marker::PhantomData, prelude::*};
use xcm::v3::prelude::*;
use xcm_executor::traits::{ConvertLocation, ExportXcm};

pub struct EthereumBlobExporter<
	UniversalLocation,
	EthereumNetwork,
	OutboundQueue,
	AgentHashedDescription,
>(PhantomData<(UniversalLocation, EthereumNetwork, OutboundQueue, AgentHashedDescription)>);

impl<UniversalLocation, EthereumNetwork, OutboundQueue, AgentHashedDescription> ExportXcm
	for EthereumBlobExporter<UniversalLocation, EthereumNetwork, OutboundQueue, AgentHashedDescription>
where
	UniversalLocation: Get<InteriorMultiLocation>,
	EthereumNetwork: Get<NetworkId>,
	OutboundQueue: SendMessage<Balance = u128>,
	OutboundQueue::Ticket: Encode + Decode,
	AgentHashedDescription: ConvertLocation<H256>,
{
	type Ticket = (Vec<u8>, XcmHash);

	fn validate(
		network: NetworkId,
		_channel: u32,
		universal_source: &mut Option<InteriorMultiLocation>,
		destination: &mut Option<InteriorMultiLocation>,
		message: &mut Option<Xcm<()>>,
	) -> SendResult<Self::Ticket> {
		let gateway_network = EthereumNetwork::get();
		let universal_location = UniversalLocation::get();

		if network != gateway_network {
			log::trace!(target: "xcm::ethereum_blob_exporter", "skipped due to unmatched bridge network {network:?}.");
			return Err(SendError::NotApplicable)
		}

		let dest = destination.take().ok_or(SendError::MissingArgument)?;
		if dest != Here {
			log::trace!(target: "xcm::ethereum_blob_exporter", "skipped due to unmatched remote destination {dest:?}.");
			return Err(SendError::NotApplicable)
		}

		let (local_net, local_sub) = universal_source
			.take()
			.ok_or_else(|| {
				log::error!(target: "xcm::ethereum_blob_exporter", "universal source not provided.");
				SendError::MissingArgument
			})?
			.split_global()
			.map_err(|()| {
				log::error!(target: "xcm::ethereum_blob_exporter", "could not get global consensus from universal source '{universal_source:?}'.");
				SendError::Unroutable
			})?;

		if Ok(local_net) != universal_location.global_consensus() {
			log::trace!(target: "xcm::ethereum_blob_exporter", "skipped due to unmatched relay network {local_net:?}.");
			return Err(SendError::NotApplicable)
		}

		let para_id = match local_sub {
			X1(Parachain(para_id)) => para_id,
			_ => {
				log::error!(target: "xcm::ethereum_blob_exporter", "could not get parachain id from universal source '{local_sub:?}'.");
				return Err(SendError::MissingArgument)
			},
		};

		let message = message.take().ok_or_else(|| {
			log::error!(target: "xcm::ethereum_blob_exporter", "xcm message not provided.");
			SendError::MissingArgument
		})?;

		let mut converter = XcmConverter::new(&message, &gateway_network);
		let agent_execute_command = converter.convert().map_err(|err|{
			log::error!(target: "xcm::ethereum_blob_exporter", "unroutable due to pattern matching error '{err:?}'.");
			SendError::Unroutable
		})?;

		// local_sub is relative to the relaychain. No conversion needed.
		let local_sub_location: MultiLocation = local_sub.into();
		let agent_id = match AgentHashedDescription::convert_location(&local_sub_location) {
			Some(id) => id,
			None => {
				log::error!(target: "xcm::ethereum_blob_exporter", "unroutable due to not being able to create agent id. '{local_sub_location:?}'");
				return Err(SendError::Unroutable)
			},
		};

		let outbound_message = Message {
			origin: para_id.into(),
			command: Command::AgentExecute { agent_id, command: agent_execute_command },
		};

		// validate the message
		let (ticket, fee) = OutboundQueue::validate(&outbound_message).map_err(|err| {
			log::error!(target: "xcm::ethereum_blob_exporter", "OutboundQueue validation of message failed. {err:?}");
			SendError::Unroutable
		})?;

		// convert fee to MultiAsset
		let fee = MultiAsset::from((MultiLocation::parent(), fee.total())).into();

		Ok(((ticket.encode(), XcmHash::default()), fee))
	}

	fn deliver(blob: (Vec<u8>, XcmHash)) -> Result<XcmHash, SendError> {
		let ticket: OutboundQueue::Ticket = OutboundQueue::Ticket::decode(&mut blob.0.as_ref())
			.map_err(|_| {
				log::trace!(target: "xcm::ethereum_blob_exporter", "undeliverable due to decoding error");
				SendError::NotApplicable
			})?;

		let message_hash = OutboundQueue::deliver(ticket).map_err(|_| {
			log::error!(target: "xcm::ethereum_blob_exporter", "OutboundQueue submit of message failed");
			SendError::Transport("other transport error")
		})?;

		log::info!(target: "xcm::ethereum_blob_exporter", "message delivered {message_hash:#?}.");
		Ok(message_hash.into())
	}
}

/// Errors that can be thrown to the pattern matching step.
#[derive(PartialEq, Debug)]
enum XcmConverterError {
	UnexpectedEndOfXcm,
	BuyExecutionExpected,
	EndOfXcmMessageExpected,
	WithdrawExpected,
	DepositAssetExpected,
	NoReserveAssets,
	ClearOriginExpected,
	FilterDoesNotConsumeAllAssets,
	TooManyAssets,
	ZeroAssetTransfer,
	BeneficiaryResolutionFailed,
	AssetResolutionFailed,
	InvalidFeeAsset,
	SetTopicExpected,
}

macro_rules! match_expression {
	($expression:expr, $(|)? $( $pattern:pat_param )|+ $( if $guard: expr )?, $value:expr $(,)?) => {
		match $expression {
			$( $pattern )|+ $( if $guard )? => Some($value),
			_ => None,
		}
	};
}

struct XcmConverter<'a, Call> {
	iter: Iter<'a, Instruction<Call>>,
	ethereum_network: &'a NetworkId,
}
impl<'a, Call> XcmConverter<'a, Call> {
	fn new(message: &'a Xcm<Call>, ethereum_network: &'a NetworkId) -> Self {
		Self { iter: message.inner().iter(), ethereum_network }
	}

	fn convert(&mut self) -> Result<AgentExecuteCommand, XcmConverterError> {
		// Get withdraw/deposit and make native tokens create message.
		let result = self.native_tokens_unlock_message()?;

		// Match last set topic. Later could use message id for replies
		let _ = match self.next()? {
			SetTopic(id) => Some(id),
			_ => return Err(XcmConverterError::SetTopicExpected),
		};

		// All xcm instructions must be consumed before exit.
		if self.next().is_ok() {
			return Err(XcmConverterError::EndOfXcmMessageExpected)
		}

		Ok(result)
	}

	fn native_tokens_unlock_message(&mut self) -> Result<AgentExecuteCommand, XcmConverterError> {
		use XcmConverterError::*;

		// Get the reserve assets from WithdrawAsset.
		let reserve_assets =
			match_expression!(self.next()?, WithdrawAsset(reserve_assets), reserve_assets)
				.ok_or(WithdrawExpected)?;

		// Check origin is cleared.
		let _ = match_expression!(self.next()?, ClearOrigin, ()).ok_or(ClearOriginExpected)?;

		// Get the fee asset from BuyExecution.
		let fee_asset = match_expression!(self.next()?, BuyExecution { fees, .. }, fees)
			.ok_or(BuyExecutionExpected)?;

		let (deposit_assets, beneficiary) = match_expression!(
			self.next()?,
			DepositAsset { assets, beneficiary },
			(assets, beneficiary)
		)
		.ok_or(DepositAssetExpected)?;

		// assert that the beneficiary is AccountKey20.
		let recipient = match_expression!(
			beneficiary,
			MultiLocation { parents: 0, interior: X1(AccountKey20 { network, key }) }
				if self.network_matches(network),
			H160(*key)
		)
		.ok_or(BeneficiaryResolutionFailed)?;

		// Make sure there are reserved assets.
		if reserve_assets.len() == 0 {
			return Err(NoReserveAssets)
		}

		// Check the the deposit asset filter matches what was reserved.
		if reserve_assets.inner().iter().any(|asset| !deposit_assets.matches(asset)) {
			return Err(FilterDoesNotConsumeAllAssets)
		}

		// We only support a single asset at a time.
		ensure!(reserve_assets.len() == 1, TooManyAssets);
		let reserve_asset = reserve_assets.get(0).ok_or(AssetResolutionFailed)?;

		// The fee asset must be the same as the reserve asset.
		if fee_asset.id != reserve_asset.id || fee_asset.fun > reserve_asset.fun {
			return Err(InvalidFeeAsset)
		}

		let (token, amount) = match_expression!(
			reserve_asset,
			MultiAsset {
				id: Concrete(MultiLocation { parents: 0, interior: X1(AccountKey20 { network , key })}),
				fun: Fungible(amount)
			} if self.network_matches(network),
			(H160(*key), *amount)
		)
		.ok_or(AssetResolutionFailed)?;

		// transfer amount must be greater than 0.
		ensure!(amount > 0, ZeroAssetTransfer);

		Ok(AgentExecuteCommand::TransferToken { token, recipient, amount })
	}

	fn next(&mut self) -> Result<&'a Instruction<Call>, XcmConverterError> {
		self.iter.next().ok_or(XcmConverterError::UnexpectedEndOfXcm)
	}

	fn network_matches(&self, network: &Option<NetworkId>) -> bool {
		if let Some(network) = network {
			network == self.ethereum_network
		} else {
			true
		}
	}
}

#[cfg(test)]
mod tests {
	use frame_support::parameter_types;
	use hex_literal::hex;
	use snowbridge_core::outbound::{Fee, SendError};
	use xcm::v3::prelude::SendError as XcmSendError;
	use xcm_builder::{DescribeAllTerminal, DescribeFamily, HashedDescription};

	pub type AgentIdOf = HashedDescription<H256, DescribeFamily<DescribeAllTerminal>>;

	use super::*;

	parameter_types! {
		const RelayNetwork: NetworkId = Polkadot;
		const UniversalLocation: InteriorMultiLocation = X2(GlobalConsensus(RelayNetwork::get()), Parachain(1013));
		const BridgedNetwork: NetworkId =  Ethereum{ chain_id: 1 };
		const NonBridgedNetwork: NetworkId =  Ethereum{ chain_id: 2 };
	}

	struct MockOkOutboundQueue;
	impl SendMessage for MockOkOutboundQueue {
		type Ticket = ();
		type Balance = u128;

		fn validate(_: &Message) -> Result<((), Fee<Self::Balance>), SendError> {
			Ok(((), Fee { local: 1, remote: 1 }))
		}

		fn deliver(_: Self::Ticket) -> Result<H256, SendError> {
			Ok(H256::zero())
		}
	}
	struct MockErrOutboundQueue;
	impl SendMessage for MockErrOutboundQueue {
		type Ticket = ();
		type Balance = u128;

		fn validate(_: &Message) -> Result<((), Fee<Self::Balance>), SendError> {
			Err(SendError::MessageTooLarge)
		}

		fn deliver(_: Self::Ticket) -> Result<H256, SendError> {
			Err(SendError::MessageTooLarge)
		}
	}

	#[test]
	fn exporter_validate_with_unknown_network_yields_not_applicable() {
		let network = Ethereum { chain_id: 1337 };
		let channel: u32 = 0;
		let mut universal_source: Option<InteriorMultiLocation> = None;
		let mut destination: Option<InteriorMultiLocation> = None;
		let mut message: Option<Xcm<()>> = None;

		let result = EthereumBlobExporter::<
			UniversalLocation,
			BridgedNetwork,
			MockOkOutboundQueue,
			AgentIdOf,
		>::validate(
			network, channel, &mut universal_source, &mut destination, &mut message
		);
		assert_eq!(result, Err(XcmSendError::NotApplicable));
	}

	#[test]
	fn exporter_validate_with_invalid_destination_yields_missing_argument() {
		let network = BridgedNetwork::get();
		let channel: u32 = 0;
		let mut universal_source: Option<InteriorMultiLocation> = None;
		let mut destination: Option<InteriorMultiLocation> = None;
		let mut message: Option<Xcm<()>> = None;

		let result = EthereumBlobExporter::<
			UniversalLocation,
			BridgedNetwork,
			MockOkOutboundQueue,
			AgentIdOf,
		>::validate(
			network, channel, &mut universal_source, &mut destination, &mut message
		);
		assert_eq!(result, Err(XcmSendError::MissingArgument));
	}

	#[test]
	fn exporter_validate_with_x8_destination_yields_not_applicable() {
		let network = BridgedNetwork::get();
		let channel: u32 = 0;
		let mut universal_source: Option<InteriorMultiLocation> = None;
		let mut destination: Option<InteriorMultiLocation> = Some(X8(
			OnlyChild, OnlyChild, OnlyChild, OnlyChild, OnlyChild, OnlyChild, OnlyChild, OnlyChild,
		));
		let mut message: Option<Xcm<()>> = None;

		let result = EthereumBlobExporter::<
			UniversalLocation,
			BridgedNetwork,
			MockOkOutboundQueue,
			AgentIdOf,
		>::validate(
			network, channel, &mut universal_source, &mut destination, &mut message
		);
		assert_eq!(result, Err(XcmSendError::NotApplicable));
	}

	#[test]
	fn exporter_validate_without_universal_source_yields_missing_argument() {
		let network = BridgedNetwork::get();
		let channel: u32 = 0;
		let mut universal_source: Option<InteriorMultiLocation> = None;
		let mut destination: Option<InteriorMultiLocation> = Here.into();
		let mut message: Option<Xcm<()>> = None;

		let result = EthereumBlobExporter::<
			UniversalLocation,
			BridgedNetwork,
			MockOkOutboundQueue,
			AgentIdOf,
		>::validate(
			network, channel, &mut universal_source, &mut destination, &mut message
		);
		assert_eq!(result, Err(XcmSendError::MissingArgument));
	}

	#[test]
	fn exporter_validate_without_global_universal_location_yields_unroutable() {
		let network = BridgedNetwork::get();
		let channel: u32 = 0;
		let mut universal_source: Option<InteriorMultiLocation> = Here.into();
		let mut destination: Option<InteriorMultiLocation> = Here.into();
		let mut message: Option<Xcm<()>> = None;

		let result = EthereumBlobExporter::<
			UniversalLocation,
			BridgedNetwork,
			MockOkOutboundQueue,
			AgentIdOf,
		>::validate(
			network, channel, &mut universal_source, &mut destination, &mut message
		);
		assert_eq!(result, Err(XcmSendError::Unroutable));
	}

	#[test]
	fn exporter_validate_without_global_bridge_location_yields_not_applicable() {
		let network = NonBridgedNetwork::get();
		let channel: u32 = 0;
		let mut universal_source: Option<InteriorMultiLocation> = Here.into();
		let mut destination: Option<InteriorMultiLocation> = Here.into();
		let mut message: Option<Xcm<()>> = None;

		let result = EthereumBlobExporter::<
			UniversalLocation,
			BridgedNetwork,
			MockOkOutboundQueue,
			AgentIdOf,
		>::validate(
			network, channel, &mut universal_source, &mut destination, &mut message
		);
		assert_eq!(result, Err(XcmSendError::NotApplicable));
	}

	#[test]
	fn exporter_validate_with_remote_universal_source_yields_not_applicable() {
		let network = BridgedNetwork::get();
		let channel: u32 = 0;
		let mut universal_source: Option<InteriorMultiLocation> =
			Some(X2(GlobalConsensus(Kusama), Parachain(1000)));
		let mut destination: Option<InteriorMultiLocation> = Here.into();
		let mut message: Option<Xcm<()>> = None;

		let result = EthereumBlobExporter::<
			UniversalLocation,
			BridgedNetwork,
			MockOkOutboundQueue,
			AgentIdOf,
		>::validate(
			network, channel, &mut universal_source, &mut destination, &mut message
		);
		assert_eq!(result, Err(XcmSendError::NotApplicable));
	}

	#[test]
	fn exporter_validate_without_para_id_in_source_yields_missing_argument() {
		let network = BridgedNetwork::get();
		let channel: u32 = 0;
		let mut universal_source: Option<InteriorMultiLocation> =
			Some(X1(GlobalConsensus(Polkadot)));
		let mut destination: Option<InteriorMultiLocation> = Here.into();
		let mut message: Option<Xcm<()>> = None;

		let result = EthereumBlobExporter::<
			UniversalLocation,
			BridgedNetwork,
			MockOkOutboundQueue,
			AgentIdOf,
		>::validate(
			network, channel, &mut universal_source, &mut destination, &mut message
		);
		assert_eq!(result, Err(XcmSendError::MissingArgument));
	}

	#[test]
	fn exporter_validate_complex_para_id_in_source_yields_missing_argument() {
		let network = BridgedNetwork::get();
		let channel: u32 = 0;
		let mut universal_source: Option<InteriorMultiLocation> =
			Some(X3(GlobalConsensus(Polkadot), Parachain(1000), PalletInstance(12)));
		let mut destination: Option<InteriorMultiLocation> = Here.into();
		let mut message: Option<Xcm<()>> = None;

		let result = EthereumBlobExporter::<
			UniversalLocation,
			BridgedNetwork,
			MockOkOutboundQueue,
			AgentIdOf,
		>::validate(
			network, channel, &mut universal_source, &mut destination, &mut message
		);
		assert_eq!(result, Err(XcmSendError::MissingArgument));
	}

	#[test]
	fn exporter_validate_without_xcm_message_yields_missing_argument() {
		let network = BridgedNetwork::get();
		let channel: u32 = 0;
		let mut universal_source: Option<InteriorMultiLocation> =
			Some(X2(GlobalConsensus(Polkadot), Parachain(1000)));
		let mut destination: Option<InteriorMultiLocation> = Here.into();
		let mut message: Option<Xcm<()>> = None;

		let result = EthereumBlobExporter::<
			UniversalLocation,
			BridgedNetwork,
			MockOkOutboundQueue,
			AgentIdOf,
		>::validate(
			network, channel, &mut universal_source, &mut destination, &mut message
		);
		assert_eq!(result, Err(XcmSendError::MissingArgument));
	}

	#[test]
	fn exporter_validate_with_max_target_fee_yields_unroutable() {
		let network = BridgedNetwork::get();
		let mut destination: Option<InteriorMultiLocation> = Here.into();

		let mut universal_source: Option<InteriorMultiLocation> =
			Some(X2(GlobalConsensus(Polkadot), Parachain(1000)));

		let token_address: [u8; 20] = hex!("1000000000000000000000000000000000000000");
		let beneficiary_address: [u8; 20] = hex!("2000000000000000000000000000000000000000");

		let channel: u32 = 0;
		let fee = MultiAsset { id: Concrete(Here.into()), fun: Fungible(1000) };
		let fees: MultiAssets = vec![fee.clone()].into();
		let assets: MultiAssets = vec![MultiAsset {
			id: Concrete(X1(AccountKey20 { network: None, key: token_address }).into()),
			fun: Fungible(1000),
		}]
		.into();
		let filter: MultiAssetFilter = assets.clone().into();

		let mut message: Option<Xcm<()>> = Some(
			vec![
				WithdrawAsset(fees),
				BuyExecution { fees: fee, weight_limit: Unlimited },
				WithdrawAsset(assets),
				DepositAsset {
					assets: filter,
					beneficiary: X1(AccountKey20 {
						network: Some(network),
						key: beneficiary_address,
					})
					.into(),
				},
				SetTopic([0; 32]),
			]
			.into(),
		);

		let result = EthereumBlobExporter::<
			UniversalLocation,
			BridgedNetwork,
			MockOkOutboundQueue,
			AgentIdOf,
		>::validate(
			network, channel, &mut universal_source, &mut destination, &mut message
		);

		assert_eq!(result, Err(XcmSendError::Unroutable));
	}

	#[test]
	fn exporter_validate_with_unparsable_xcm_yields_unroutable() {
		let network = BridgedNetwork::get();
		let mut destination: Option<InteriorMultiLocation> = Here.into();

		let mut universal_source: Option<InteriorMultiLocation> =
			Some(X2(GlobalConsensus(Polkadot), Parachain(1000)));

		let channel: u32 = 0;
		let fee = MultiAsset { id: Concrete(Here.into()), fun: Fungible(1000) };
		let fees: MultiAssets = vec![fee.clone()].into();

		let mut message: Option<Xcm<()>> = Some(
			vec![WithdrawAsset(fees), BuyExecution { fees: fee, weight_limit: Unlimited }].into(),
		);

		let result = EthereumBlobExporter::<
			UniversalLocation,
			BridgedNetwork,
			MockOkOutboundQueue,
			AgentIdOf,
		>::validate(
			network, channel, &mut universal_source, &mut destination, &mut message
		);

		assert_eq!(result, Err(XcmSendError::Unroutable));
	}

	#[test]
	fn exporter_validate_xcm_success_case_1() {
		let network = BridgedNetwork::get();
		let mut destination: Option<InteriorMultiLocation> = Here.into();

		let mut universal_source: Option<InteriorMultiLocation> =
			Some(X2(GlobalConsensus(Polkadot), Parachain(1000)));

		let token_address: [u8; 20] = hex!("1000000000000000000000000000000000000000");
		let beneficiary_address: [u8; 20] = hex!("2000000000000000000000000000000000000000");

		let channel: u32 = 0;
		let assets: MultiAssets = vec![MultiAsset {
			id: Concrete(X1(AccountKey20 { network: None, key: token_address }).into()),
			fun: Fungible(1000),
		}]
		.into();
		let fee = assets.clone().get(0).unwrap().clone();
		let filter: MultiAssetFilter = assets.clone().into();

		let mut message: Option<Xcm<()>> = Some(
			vec![
				WithdrawAsset(assets.clone()),
				ClearOrigin,
				BuyExecution { fees: fee, weight_limit: Unlimited },
				DepositAsset {
					assets: filter,
					beneficiary: X1(AccountKey20 { network: None, key: beneficiary_address })
						.into(),
				},
				SetTopic([0; 32]),
			]
			.into(),
		);

		let result = EthereumBlobExporter::<
			UniversalLocation,
			BridgedNetwork,
			MockOkOutboundQueue,
			AgentIdOf,
		>::validate(
			network, channel, &mut universal_source, &mut destination, &mut message
		);

		assert!(result.is_ok());
	}

	#[test]
	fn exporter_deliver_with_submit_failure_yields_unroutable() {
		let result = EthereumBlobExporter::<
			UniversalLocation,
			BridgedNetwork,
			MockErrOutboundQueue,
			AgentIdOf,
		>::deliver((hex!("deadbeef").to_vec(), XcmHash::default()));
		assert_eq!(result, Err(XcmSendError::Transport("other transport error")))
	}

	#[test]
	fn xcm_converter_convert_success() {
		let network = BridgedNetwork::get();

		let token_address: [u8; 20] = hex!("1000000000000000000000000000000000000000");
		let beneficiary_address: [u8; 20] = hex!("2000000000000000000000000000000000000000");

		let assets: MultiAssets = vec![MultiAsset {
			id: Concrete(X1(AccountKey20 { network: None, key: token_address }).into()),
			fun: Fungible(1000),
		}]
		.into();
		let filter: MultiAssetFilter = assets.clone().into();

		let message: Xcm<()> = vec![
			WithdrawAsset(assets.clone()),
			ClearOrigin,
			BuyExecution { fees: assets.get(0).unwrap().clone(), weight_limit: Unlimited },
			DepositAsset {
				assets: filter,
				beneficiary: X1(AccountKey20 { network: None, key: beneficiary_address }).into(),
			},
			SetTopic([0; 32]),
		]
		.into();
		let mut converter = XcmConverter::new(&message, &network);
		let expected_payload = AgentExecuteCommand::TransferToken {
			token: token_address.into(),
			recipient: beneficiary_address.into(),
			amount: 1000,
		};
		let result = converter.convert();
		assert_eq!(result, Ok(expected_payload));
	}

	#[test]
	fn xcm_converter_convert_with_wildcard_all_asset_filter_succeeds() {
		let network = BridgedNetwork::get();

		let token_address: [u8; 20] = hex!("1000000000000000000000000000000000000000");
		let beneficiary_address: [u8; 20] = hex!("2000000000000000000000000000000000000000");

		let assets: MultiAssets = vec![MultiAsset {
			id: Concrete(X1(AccountKey20 { network: None, key: token_address }).into()),
			fun: Fungible(1000),
		}]
		.into();
		let filter: MultiAssetFilter = Wild(All);

		let message: Xcm<()> = vec![
			WithdrawAsset(assets.clone()),
			ClearOrigin,
			BuyExecution { fees: assets.get(0).unwrap().clone(), weight_limit: Unlimited },
			DepositAsset {
				assets: filter,
				beneficiary: X1(AccountKey20 { network: None, key: beneficiary_address }).into(),
			},
			SetTopic([0; 32]),
		]
		.into();
		let mut converter = XcmConverter::new(&message, &network);
		let expected_payload = AgentExecuteCommand::TransferToken {
			token: token_address.into(),
			recipient: beneficiary_address.into(),
			amount: 1000,
		};
		let result = converter.convert();
		assert_eq!(result, Ok(expected_payload));
	}

	#[test]
	fn xcm_converter_convert_with_fees_less_than_reserve_yields_success() {
		let network = BridgedNetwork::get();

		let token_address: [u8; 20] = hex!("1000000000000000000000000000000000000000");
		let beneficiary_address: [u8; 20] = hex!("2000000000000000000000000000000000000000");

		let asset_location = X1(AccountKey20 { network: None, key: token_address }).into();
		let fee_asset = MultiAsset { id: Concrete(asset_location), fun: Fungible(500) };

		let assets: MultiAssets =
			vec![MultiAsset { id: Concrete(asset_location), fun: Fungible(1000) }].into();

		let filter: MultiAssetFilter = assets.clone().into();

		let message: Xcm<()> = vec![
			WithdrawAsset(assets.clone()),
			ClearOrigin,
			BuyExecution { fees: fee_asset, weight_limit: Unlimited },
			DepositAsset {
				assets: filter,
				beneficiary: X1(AccountKey20 { network: None, key: beneficiary_address }).into(),
			},
			SetTopic([0; 32]),
		]
		.into();
		let mut converter = XcmConverter::new(&message, &network);
		let expected_payload = AgentExecuteCommand::TransferToken {
			token: token_address.into(),
			recipient: beneficiary_address.into(),
			amount: 1000,
		};
		let result = converter.convert();
		assert_eq!(result, Ok(expected_payload));
	}

	#[test]
	fn xcm_converter_convert_with_partial_message_yields_unexpected_end_of_xcm() {
		let network = BridgedNetwork::get();

		let token_address: [u8; 20] = hex!("1000000000000000000000000000000000000000");
		let assets: MultiAssets = vec![MultiAsset {
			id: Concrete(X1(AccountKey20 { network: None, key: token_address }).into()),
			fun: Fungible(1000),
		}]
		.into();
		let message: Xcm<()> = vec![WithdrawAsset(assets)].into();

		let mut converter = XcmConverter::new(&message, &network);
		let result = converter.convert();
		assert_eq!(result.err(), Some(XcmConverterError::UnexpectedEndOfXcm));
	}

	#[test]
	fn xcm_converter_with_different_fee_asset_fails() {
		let network = BridgedNetwork::get();

		let token_address: [u8; 20] = hex!("1000000000000000000000000000000000000000");
		let beneficiary_address: [u8; 20] = hex!("2000000000000000000000000000000000000000");

		let asset_location = X1(AccountKey20 { network: None, key: token_address }).into();
		let fee_asset = MultiAsset {
			id: Concrete(MultiLocation { parents: 0, interior: Here.into() }),
			fun: Fungible(1000),
		};

		let assets: MultiAssets =
			vec![MultiAsset { id: Concrete(asset_location), fun: Fungible(1000) }].into();

		let filter: MultiAssetFilter = assets.clone().into();

		let message: Xcm<()> = vec![
			WithdrawAsset(assets.clone()),
			ClearOrigin,
			BuyExecution { fees: fee_asset, weight_limit: Unlimited },
			DepositAsset {
				assets: filter,
				beneficiary: X1(AccountKey20 { network: None, key: beneficiary_address }).into(),
			},
			SetTopic([0; 32]),
		]
		.into();
		let mut converter = XcmConverter::new(&message, &network);
		let result = converter.convert();
		assert_eq!(result.err(), Some(XcmConverterError::InvalidFeeAsset));
	}

	#[test]
	fn xcm_converter_with_fees_greater_than_reserve_fails() {
		let network = BridgedNetwork::get();

		let token_address: [u8; 20] = hex!("1000000000000000000000000000000000000000");
		let beneficiary_address: [u8; 20] = hex!("2000000000000000000000000000000000000000");

		let asset_location = X1(AccountKey20 { network: None, key: token_address }).into();
		let fee_asset = MultiAsset { id: Concrete(asset_location), fun: Fungible(1001) };

		let assets: MultiAssets =
			vec![MultiAsset { id: Concrete(asset_location), fun: Fungible(1000) }].into();

		let filter: MultiAssetFilter = assets.clone().into();

		let message: Xcm<()> = vec![
			WithdrawAsset(assets.clone()),
			ClearOrigin,
			BuyExecution { fees: fee_asset, weight_limit: Unlimited },
			DepositAsset {
				assets: filter,
				beneficiary: X1(AccountKey20 { network: None, key: beneficiary_address }).into(),
			},
			SetTopic([0; 32]),
		]
		.into();
		let mut converter = XcmConverter::new(&message, &network);
		let result = converter.convert();
		assert_eq!(result.err(), Some(XcmConverterError::InvalidFeeAsset));
	}

	#[test]
	fn xcm_converter_convert_without_clear_origin_fails() {
		let network = BridgedNetwork::get();

		let token_address: [u8; 20] = hex!("1000000000000000000000000000000000000000");
		let beneficiary_address: [u8; 20] = hex!("2000000000000000000000000000000000000000");

		let assets: MultiAssets = vec![MultiAsset {
			id: Concrete(X1(AccountKey20 { network: None, key: token_address }).into()),
			fun: Fungible(1000),
		}]
		.into();
		let filter: MultiAssetFilter = assets.clone().into();

		let message: Xcm<()> = vec![
			WithdrawAsset(assets.clone()),
			BuyExecution { fees: assets.get(0).unwrap().clone(), weight_limit: Unlimited },
			DepositAsset {
				assets: filter,
				beneficiary: X1(AccountKey20 { network: None, key: beneficiary_address }).into(),
			},
			SetTopic([0; 32]),
		]
		.into();
		let mut converter = XcmConverter::new(&message, &network);
		let result = converter.convert();
		assert_eq!(result.err(), Some(XcmConverterError::ClearOriginExpected));
	}

	#[test]
	fn xcm_converter_convert_with_empty_xcm_yields_unexpected_end_of_xcm() {
		let network = BridgedNetwork::get();

		let message: Xcm<()> = vec![].into();

		let mut converter = XcmConverter::new(&message, &network);

		let result = converter.convert();
		assert_eq!(result.err(), Some(XcmConverterError::UnexpectedEndOfXcm));
	}

	#[test]
	fn xcm_converter_convert_without_set_topic_suffix_yields_set_topic_expected() {
		let network = BridgedNetwork::get();

		let token_address: [u8; 20] = hex!("1000000000000000000000000000000000000000");
		let beneficiary_address: [u8; 20] = hex!("2000000000000000000000000000000000000000");

		let assets: MultiAssets = vec![MultiAsset {
			id: Concrete(X1(AccountKey20 { network: None, key: token_address }).into()),
			fun: Fungible(1000),
		}]
		.into();
		let filter: MultiAssetFilter = assets.clone().into();

		let message: Xcm<()> = vec![
			WithdrawAsset(assets.clone()),
			ClearOrigin,
			BuyExecution { fees: assets.get(0).unwrap().clone(), weight_limit: Unlimited },
			DepositAsset {
				assets: filter,
				beneficiary: X1(AccountKey20 { network: None, key: beneficiary_address }).into(),
			},
			ClearTopic,
		]
		.into();
		let mut converter = XcmConverter::new(&message, &network);

		let result = converter.convert();
		assert_eq!(result.err(), Some(XcmConverterError::SetTopicExpected));
	}

	#[test]
	fn xcm_converter_convert_with_extra_instructions_yields_end_of_xcm_message_expected() {
		let network = BridgedNetwork::get();

		let token_address: [u8; 20] = hex!("1000000000000000000000000000000000000000");
		let beneficiary_address: [u8; 20] = hex!("2000000000000000000000000000000000000000");

		let assets: MultiAssets = vec![MultiAsset {
			id: Concrete(X1(AccountKey20 { network: None, key: token_address }).into()),
			fun: Fungible(1000),
		}]
		.into();
		let filter: MultiAssetFilter = assets.clone().into();

		let message: Xcm<()> = vec![
			WithdrawAsset(assets.clone()),
			ClearOrigin,
			BuyExecution { fees: assets.get(0).unwrap().clone(), weight_limit: Unlimited },
			DepositAsset {
				assets: filter,
				beneficiary: X1(AccountKey20 { network: None, key: beneficiary_address }).into(),
			},
			SetTopic([0; 32]),
			ClearError,
		]
		.into();
		let mut converter = XcmConverter::new(&message, &network);

		let result = converter.convert();
		assert_eq!(result.err(), Some(XcmConverterError::EndOfXcmMessageExpected));
	}

	#[test]
	fn xcm_converter_convert_without_withdraw_asset_yields_withdraw_expected() {
		let network = BridgedNetwork::get();

		let token_address: [u8; 20] = hex!("1000000000000000000000000000000000000000");
		let beneficiary_address: [u8; 20] = hex!("2000000000000000000000000000000000000000");

		let assets: MultiAssets = vec![MultiAsset {
			id: Concrete(X1(AccountKey20 { network: None, key: token_address }).into()),
			fun: Fungible(1000),
		}]
		.into();
		let filter: MultiAssetFilter = assets.clone().into();

		let message: Xcm<()> = vec![
			ClearOrigin,
			BuyExecution { fees: assets.get(0).unwrap().clone(), weight_limit: Unlimited },
			DepositAsset {
				assets: filter,
				beneficiary: X1(AccountKey20 { network: None, key: beneficiary_address }).into(),
			},
			SetTopic([0; 32]),
		]
		.into();
		let mut converter = XcmConverter::new(&message, &network);

		let result = converter.convert();
		assert_eq!(result.err(), Some(XcmConverterError::WithdrawExpected));
	}

	#[test]
	fn xcm_converter_convert_without_withdraw_asset_yields_deposit_expected() {
		let network = BridgedNetwork::get();

		let token_address: [u8; 20] = hex!("1000000000000000000000000000000000000000");

		let assets: MultiAssets = vec![MultiAsset {
			id: Concrete(X1(AccountKey20 { network: None, key: token_address }).into()),
			fun: Fungible(1000),
		}]
		.into();

		let message: Xcm<()> = vec![
			WithdrawAsset(assets.clone()),
			ClearOrigin,
			BuyExecution { fees: assets.get(0).unwrap().clone(), weight_limit: Unlimited },
			SetTopic([0; 32]),
		]
		.into();
		let mut converter = XcmConverter::new(&message, &network);

		let result = converter.convert();
		assert_eq!(result.err(), Some(XcmConverterError::DepositAssetExpected));
	}

	#[test]
	fn xcm_converter_convert_without_assets_yields_no_reserve_assets() {
		let network = BridgedNetwork::get();

		let token_address: [u8; 20] = hex!("1000000000000000000000000000000000000000");

		let beneficiary_address: [u8; 20] = hex!("2000000000000000000000000000000000000000");

		let assets: MultiAssets = vec![].into();
		let filter: MultiAssetFilter = assets.clone().into();

		let fee = MultiAsset {
			id: Concrete(X1(AccountKey20 { network: None, key: token_address }).into()),
			fun: Fungible(1000),
		};

		let message: Xcm<()> = vec![
			WithdrawAsset(assets.clone()),
			ClearOrigin,
			BuyExecution { fees: fee, weight_limit: Unlimited },
			DepositAsset {
				assets: filter,
				beneficiary: X1(AccountKey20 { network: None, key: beneficiary_address }).into(),
			},
			SetTopic([0; 32]),
		]
		.into();
		let mut converter = XcmConverter::new(&message, &network);

		let result = converter.convert();
		assert_eq!(result.err(), Some(XcmConverterError::NoReserveAssets));
	}

	#[test]
	fn xcm_converter_convert_with_two_assets_yields_too_many_assets() {
		let network = BridgedNetwork::get();

		let token_address_1: [u8; 20] = hex!("1000000000000000000000000000000000000000");
		let token_address_2: [u8; 20] = hex!("1100000000000000000000000000000000000000");
		let beneficiary_address: [u8; 20] = hex!("2000000000000000000000000000000000000000");

		let assets: MultiAssets = vec![
			MultiAsset {
				id: Concrete(X1(AccountKey20 { network: None, key: token_address_1 }).into()),
				fun: Fungible(1000),
			},
			MultiAsset {
				id: Concrete(X1(AccountKey20 { network: None, key: token_address_2 }).into()),
				fun: Fungible(500),
			},
		]
		.into();
		let filter: MultiAssetFilter = assets.clone().into();

		let message: Xcm<()> = vec![
			WithdrawAsset(assets.clone()),
			ClearOrigin,
			BuyExecution { fees: assets.get(0).unwrap().clone(), weight_limit: Unlimited },
			DepositAsset {
				assets: filter,
				beneficiary: X1(AccountKey20 { network: None, key: beneficiary_address }).into(),
			},
			SetTopic([0; 32]),
		]
		.into();
		let mut converter = XcmConverter::new(&message, &network);

		let result = converter.convert();
		assert_eq!(result.err(), Some(XcmConverterError::TooManyAssets));
	}

	#[test]
	fn xcm_converter_convert_without_consuming_filter_yields_filter_does_not_consume_all_assets() {
		let network = BridgedNetwork::get();

		let token_address: [u8; 20] = hex!("1000000000000000000000000000000000000000");
		let beneficiary_address: [u8; 20] = hex!("2000000000000000000000000000000000000000");

		let assets: MultiAssets = vec![MultiAsset {
			id: Concrete(X1(AccountKey20 { network: None, key: token_address }).into()),
			fun: Fungible(1000),
		}]
		.into();
		let filter: MultiAssetFilter = Wild(WildMultiAsset::AllCounted(0));

		let message: Xcm<()> = vec![
			WithdrawAsset(assets.clone()),
			ClearOrigin,
			BuyExecution { fees: assets.get(0).unwrap().clone(), weight_limit: Unlimited },
			DepositAsset {
				assets: filter,
				beneficiary: X1(AccountKey20 { network: None, key: beneficiary_address }).into(),
			},
			SetTopic([0; 32]),
		]
		.into();
		let mut converter = XcmConverter::new(&message, &network);

		let result = converter.convert();
		assert_eq!(result.err(), Some(XcmConverterError::FilterDoesNotConsumeAllAssets));
	}

	#[test]
	fn xcm_converter_convert_with_zero_amount_asset_yields_zero_asset_transfer() {
		let network = BridgedNetwork::get();

		let token_address: [u8; 20] = hex!("1000000000000000000000000000000000000000");
		let beneficiary_address: [u8; 20] = hex!("2000000000000000000000000000000000000000");

		let assets: MultiAssets = vec![MultiAsset {
			id: Concrete(X1(AccountKey20 { network: None, key: token_address }).into()),
			fun: Fungible(0),
		}]
		.into();
		let filter: MultiAssetFilter = Wild(WildMultiAsset::AllCounted(1));

		let message: Xcm<()> = vec![
			WithdrawAsset(assets.clone()),
			ClearOrigin,
			BuyExecution { fees: assets.get(0).unwrap().clone(), weight_limit: Unlimited },
			DepositAsset {
				assets: filter,
				beneficiary: X1(AccountKey20 { network: None, key: beneficiary_address }).into(),
			},
			SetTopic([0; 32]),
		]
		.into();
		let mut converter = XcmConverter::new(&message, &network);

		let result = converter.convert();
		assert_eq!(result.err(), Some(XcmConverterError::ZeroAssetTransfer));
	}

	#[test]
	fn xcm_converter_convert_non_ethereum_asset_yields_asset_resolution_failed() {
		let network = BridgedNetwork::get();

		let beneficiary_address: [u8; 20] = hex!("2000000000000000000000000000000000000000");

		let assets: MultiAssets = vec![MultiAsset {
			id: Concrete(X3(GlobalConsensus(Polkadot), Parachain(1000), GeneralIndex(0)).into()),
			fun: Fungible(1000),
		}]
		.into();
		let filter: MultiAssetFilter = Wild(WildMultiAsset::AllCounted(1));

		let message: Xcm<()> = vec![
			WithdrawAsset(assets.clone()),
			ClearOrigin,
			BuyExecution { fees: assets.get(0).unwrap().clone(), weight_limit: Unlimited },
			DepositAsset {
				assets: filter,
				beneficiary: X1(AccountKey20 { network: None, key: beneficiary_address }).into(),
			},
			SetTopic([0; 32]),
		]
		.into();
		let mut converter = XcmConverter::new(&message, &network);

		let result = converter.convert();
		assert_eq!(result.err(), Some(XcmConverterError::AssetResolutionFailed));
	}

	#[test]
	fn xcm_converter_convert_non_ethereum_chain_asset_yields_asset_resolution_failed() {
		let network = BridgedNetwork::get();

		let token_address: [u8; 20] = hex!("1000000000000000000000000000000000000000");
		let beneficiary_address: [u8; 20] = hex!("2000000000000000000000000000000000000000");

		let assets: MultiAssets = vec![MultiAsset {
			id: Concrete(
				X1(AccountKey20 { network: Some(Ethereum { chain_id: 2 }), key: token_address })
					.into(),
			),
			fun: Fungible(1000),
		}]
		.into();
		let filter: MultiAssetFilter = Wild(WildMultiAsset::AllCounted(1));

		let message: Xcm<()> = vec![
			WithdrawAsset(assets.clone()),
			ClearOrigin,
			BuyExecution { fees: assets.get(0).unwrap().clone(), weight_limit: Unlimited },
			DepositAsset {
				assets: filter,
				beneficiary: X1(AccountKey20 { network: None, key: beneficiary_address }).into(),
			},
			SetTopic([0; 32]),
		]
		.into();
		let mut converter = XcmConverter::new(&message, &network);

		let result = converter.convert();
		assert_eq!(result.err(), Some(XcmConverterError::AssetResolutionFailed));
	}

	#[test]
	fn xcm_converter_convert_non_ethereum_chain_yields_asset_resolution_failed() {
		let network = BridgedNetwork::get();

		let token_address: [u8; 20] = hex!("1000000000000000000000000000000000000000");
		let beneficiary_address: [u8; 20] = hex!("2000000000000000000000000000000000000000");

		let assets: MultiAssets = vec![MultiAsset {
			id: Concrete(
				X1(AccountKey20 { network: Some(NonBridgedNetwork::get()), key: token_address })
					.into(),
			),
			fun: Fungible(1000),
		}]
		.into();
		let filter: MultiAssetFilter = Wild(WildMultiAsset::AllCounted(1));

		let message: Xcm<()> = vec![
			WithdrawAsset(assets.clone()),
			ClearOrigin,
			BuyExecution { fees: assets.get(0).unwrap().clone(), weight_limit: Unlimited },
			DepositAsset {
				assets: filter,
				beneficiary: X1(AccountKey20 { network: None, key: beneficiary_address }).into(),
			},
			SetTopic([0; 32]),
		]
		.into();
		let mut converter = XcmConverter::new(&message, &network);

		let result = converter.convert();
		assert_eq!(result.err(), Some(XcmConverterError::AssetResolutionFailed));
	}

	#[test]
	fn xcm_converter_convert_with_non_ethereum_beneficiary_yields_beneficiary_resolution_failed() {
		let network = BridgedNetwork::get();

		let token_address: [u8; 20] = hex!("1000000000000000000000000000000000000000");

		let beneficiary_address: [u8; 32] =
			hex!("2000000000000000000000000000000000000000000000000000000000000000");

		let assets: MultiAssets = vec![MultiAsset {
			id: Concrete(X1(AccountKey20 { network: None, key: token_address }).into()),
			fun: Fungible(1000),
		}]
		.into();
		let filter: MultiAssetFilter = Wild(WildMultiAsset::AllCounted(1));
		let message: Xcm<()> = vec![
			WithdrawAsset(assets.clone()),
			ClearOrigin,
			BuyExecution { fees: assets.get(0).unwrap().clone(), weight_limit: Unlimited },
			DepositAsset {
				assets: filter,
				beneficiary: X3(
					GlobalConsensus(Polkadot),
					Parachain(1000),
					AccountId32 { network: Some(Polkadot), id: beneficiary_address },
				)
				.into(),
			},
			SetTopic([0; 32]),
		]
		.into();
		let mut converter = XcmConverter::new(&message, &network);

		let result = converter.convert();
		assert_eq!(result.err(), Some(XcmConverterError::BeneficiaryResolutionFailed));
	}

	#[test]
	fn xcm_converter_convert_with_non_ethereum_chain_beneficiary_yields_beneficiary_resolution_failed(
	) {
		let network = BridgedNetwork::get();

		let token_address: [u8; 20] = hex!("1000000000000000000000000000000000000000");
		let beneficiary_address: [u8; 20] = hex!("2000000000000000000000000000000000000000");

		let assets: MultiAssets = vec![MultiAsset {
			id: Concrete(X1(AccountKey20 { network: None, key: token_address }).into()),
			fun: Fungible(1000),
		}]
		.into();
		let filter: MultiAssetFilter = Wild(WildMultiAsset::AllCounted(1));

		let message: Xcm<()> = vec![
			WithdrawAsset(assets.clone()),
			ClearOrigin,
			BuyExecution { fees: assets.get(0).unwrap().clone(), weight_limit: Unlimited },
			DepositAsset {
				assets: filter,
				beneficiary: X1(AccountKey20 {
					network: Some(Ethereum { chain_id: 2 }),
					key: beneficiary_address,
				})
				.into(),
			},
			SetTopic([0; 32]),
		]
		.into();
		let mut converter = XcmConverter::new(&message, &network);

		let result = converter.convert();
		assert_eq!(result.err(), Some(XcmConverterError::BeneficiaryResolutionFailed));
	}
}
