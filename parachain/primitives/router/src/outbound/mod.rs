// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
//! Converts XCM messages into simpler commands that can be processed by the Gateway contract
use core::slice::Iter;

use codec::{Decode, Encode};

use frame_support::{ensure, traits::Get};
use log;
use snowbridge_core::outbound::{
	AgentExecuteCommand, Command, Message, OutboundQueue as OutboundQueueTrait,
};
use sp_core::H256;
use sp_std::{marker::PhantomData, prelude::*};
use xcm::v3::prelude::*;
use xcm_executor::traits::{ConvertLocation, ExportXcm};

pub struct EthereumBlobExporter<
	UniversalLocation,
	GatewayLocation,
	OutboundQueue,
	AgentHashedDescription,
>(PhantomData<(UniversalLocation, GatewayLocation, OutboundQueue, AgentHashedDescription)>);

impl<UniversalLocation, GatewayLocation, OutboundQueue, AgentHashedDescription> ExportXcm
	for EthereumBlobExporter<UniversalLocation, GatewayLocation, OutboundQueue, AgentHashedDescription>
where
	UniversalLocation: Get<InteriorMultiLocation>,
	GatewayLocation: Get<MultiLocation>,
	OutboundQueue: OutboundQueueTrait<Balance = u128>,
	OutboundQueue::Ticket: Encode + Decode,
	AgentHashedDescription: ConvertLocation<H256>,
{
	type Ticket = Vec<u8>;

	fn validate(
		network: NetworkId,
		_channel: u32,
		universal_source: &mut Option<InteriorMultiLocation>,
		destination: &mut Option<InteriorMultiLocation>,
		message: &mut Option<Xcm<()>>,
	) -> SendResult<Self::Ticket> {
		let gateway_location = GatewayLocation::get();
		let universal_location = UniversalLocation::get();

		let (gateway_network, gateway_junctions) = gateway_location.interior().split_global()
			.map_err(|_| {
				log::trace!(target: "xcm::ethereum_blob_exporter", "skipped due to bridge location not being remote. universal_location: {universal_location:?}, {gateway_location:?}");
				SendError::NotApplicable
			})?;

		if network != gateway_network {
			log::trace!(target: "xcm::ethereum_blob_exporter", "skipped due to unmatched bridge network {network:?}.");
			return Err(SendError::NotApplicable);
		}

		let dest = destination.take().ok_or(SendError::MissingArgument)?;
		if dest != Here {
			log::trace!(target: "xcm::ethereum_blob_exporter", "skipped due to unmatched remote destination {dest:?}.");
			return Err(SendError::NotApplicable);
		}

		let gateway_address = match gateway_junctions {
			X1(AccountKey20 { network, key })
				if network.is_none() || network == Some(gateway_network) =>
			{
				key
			},
			_ => {
				log::trace!(target: "xcm::ethereum_blob_exporter", "skipped due to unmatched registry contract {gateway_junctions:?}.");
				return Err(SendError::NotApplicable);
			},
		};

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
			return Err(SendError::NotApplicable);
		}

		let para_id = match local_sub {
			X1(Parachain(para_id)) => para_id,
			_ => {
				log::error!(target: "xcm::ethereum_blob_exporter", "could not get parachain id from universal source '{local_sub:?}'.");
				return Err(SendError::MissingArgument);
			},
		};

		let message = message.take().ok_or_else(|| {
			log::error!(target: "xcm::ethereum_blob_exporter", "xcm message not provided.");
			SendError::MissingArgument
		})?;

		let mut converter = XcmConverter::new(&message, &gateway_network, &gateway_address);
		let (agent_execute_command, max_target_fee) = converter.convert().map_err(|err|{
			log::error!(target: "xcm::ethereum_blob_exporter", "unroutable due to pattern matching error '{err:?}'.");
			SendError::Unroutable
		})?;

		if max_target_fee.is_some() {
			log::error!(target: "xcm::ethereum_blob_exporter", "unroutable due not supporting max target fee.");
			return Err(SendError::Unroutable);
		}

		// local_sub is relative to the relaychain. No conversion needed.
		let local_sub_location: MultiLocation = local_sub.into();
		let agent_id = match AgentHashedDescription::convert_location(&local_sub_location) {
			Some(id) => id,
			None => {
				log::error!(target: "xcm::ethereum_blob_exporter", "unroutable due to not being able to create agent id. '{local_sub_location:?}'");
				return Err(SendError::Unroutable);
			},
		};

		let outbound_message = Message {
			origin: para_id.into(),
			command: Command::AgentExecute { agent_id, command: agent_execute_command },
		};

		// validate the message
		let (ticket, fees) = OutboundQueue::validate(&outbound_message).map_err(|err| {
			log::error!(target: "xcm::ethereum_blob_exporter", "OutboundQueue validation of message failed. {err:?}");
			SendError::Unroutable
		})?;

		// convert fee to MultiAsset
		let fee = MultiAsset::from((MultiLocation::parent(), fees.total())).into();

		Ok((ticket.encode(), fee))
	}

	fn deliver(blob: Vec<u8>) -> Result<XcmHash, SendError> {
		let ticket: OutboundQueue::Ticket = OutboundQueue::Ticket::decode(&mut blob.as_ref())
			.map_err(|_| {
				log::trace!(target: "xcm::ethereum_blob_exporter", "undeliverable due to decoding error");
				SendError::NotApplicable
			})?;

		let message_hash = OutboundQueue::submit(ticket).map_err(|_| {
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
	TargetFeeExpected,
	BuyExecutionExpected,
	EndOfXcmMessageExpected,
	WithdrawExpected,
	DepositExpected,
	NoReserveAssets,
	FilterDoesNotConsumeAllAssets,
	TooManyAssets,
	AssetNotConcreteFungible,
	ZeroAssetTransfer,
	BeneficiaryResolutionFailed,
	AssetResolutionFailed,
	SetTopicExpected,
}

struct XcmConverter<'a, Call> {
	iter: Iter<'a, Instruction<Call>>,
	ethereum_network: &'a NetworkId,
	gateway_address: &'a [u8; 20],
}
impl<'a, Call> XcmConverter<'a, Call> {
	fn new(
		message: &'a Xcm<Call>,
		ethereum_network: &'a NetworkId,
		gateway_address: &'a [u8; 20],
	) -> Self {
		Self { iter: message.inner().iter(), ethereum_network, gateway_address }
	}

	fn convert(
		&mut self,
	) -> Result<(AgentExecuteCommand, Option<&'a MultiAsset>), XcmConverterError> {
		// Get target fees if specified.
		let max_target_fee = self.fee_info()?;

		// Get withdraw/deposit and make native tokens create message.
		let result = self.native_tokens_unlock_message()?;

		// Match last set topic. Later could use message id for replies
		let _ = match self.next()? {
			SetTopic(id) => id,
			_ => return Err(XcmConverterError::SetTopicExpected),
		};

		// All xcm instructions must be consumed before exit.
		if self.next().is_ok() {
			return Err(XcmConverterError::EndOfXcmMessageExpected);
		}

		Ok((result, max_target_fee))
	}

	fn fee_info(&mut self) -> Result<Option<&'a MultiAsset>, XcmConverterError> {
		use XcmConverterError::*;
		let execution_fee = match self.next()? {
			WithdrawAsset(fee_asset) => match self.next()? {
				BuyExecution { fees: execution_fee, weight_limit: Unlimited }
					if fee_asset.len() == 1 && fee_asset.contains(execution_fee) =>
				{
					Some(execution_fee)
				},
				_ => return Err(BuyExecutionExpected),
			},
			UnpaidExecution { check_origin: None, weight_limit: Unlimited } => None,
			_ => return Err(TargetFeeExpected),
		};
		Ok(execution_fee)
	}

	fn native_tokens_unlock_message(&mut self) -> Result<AgentExecuteCommand, XcmConverterError> {
		use XcmConverterError::*;
		let (assets, beneficiary) = if let WithdrawAsset(reserved_assets) = self.next()? {
			if reserved_assets.len() == 0 {
				return Err(NoReserveAssets);
			}
			if let DepositAsset { assets, beneficiary } = self.next()? {
				if reserved_assets.inner().iter().any(|asset| !assets.matches(asset)) {
					return Err(FilterDoesNotConsumeAllAssets);
				}
				(reserved_assets, beneficiary)
			} else {
				return Err(DepositExpected);
			}
		} else {
			return Err(WithdrawExpected);
		};

		// assert that the beneficiary is AccountKey20
		let destination = {
			if let MultiLocation { parents: 0, interior: X1(AccountKey20 { network, key }) } =
				beneficiary
			{
				if network.is_some() && network != &Some(*self.ethereum_network) {
					return Err(BeneficiaryResolutionFailed);
				}
				key.into()
			} else {
				return Err(BeneficiaryResolutionFailed);
			}
		};

		let (asset, amount) = {
			// We only support a single asset at a time.
			ensure!(assets.len() == 1, TooManyAssets);

			// Ensure asset is concrete and fungible.
			let asset = assets.get(0).ok_or(AssetResolutionFailed)?;
			let (asset_location, amount) =
				if let MultiAsset { id: Concrete(location), fun: Fungible(amount) } = asset {
					(location, amount)
				} else {
					return Err(AssetNotConcreteFungible);
				};

			ensure!(*amount > 0, ZeroAssetTransfer);

			// extract ERC20 contract address
			if let MultiLocation {
				parents: 0,
				interior:
					X2(
						AccountKey20 { network: gateway_network, key: gateway_address },
						AccountKey20 { network: token_network, key: token_address },
					),
			} = asset_location
			{
				if gateway_network.is_some() && gateway_network != &Some(*self.ethereum_network) {
					return Err(AssetResolutionFailed);
				}
				if gateway_address != self.gateway_address {
					return Err(AssetResolutionFailed);
				}
				if token_network.is_some() && token_network != &Some(*self.ethereum_network) {
					return Err(AssetResolutionFailed);
				}
				(token_address.into(), *amount)
			} else {
				return Err(AssetResolutionFailed);
			}
		};

		Ok(AgentExecuteCommand::TransferToken { token: asset, recipient: destination, amount })
	}

	fn next(&mut self) -> Result<&'a Instruction<Call>, XcmConverterError> {
		self.iter.next().ok_or(XcmConverterError::UnexpectedEndOfXcm)
	}
}

#[cfg(test)]
mod tests {
	use frame_support::parameter_types;
	use hex_literal::hex;
	use snowbridge_core::outbound::{Fees, MessageHash, SendError};
	use xcm::v3::prelude::SendError as XcmSendError;
	use xcm_builder::{DescribeAllTerminal, DescribeFamily, HashedDescription};

	pub type AgentIdOf = HashedDescription<H256, DescribeFamily<DescribeAllTerminal>>;

	use super::*;

	parameter_types! {
		const RelayNetwork: NetworkId = Polkadot;
		const UniversalLocation: InteriorMultiLocation = X2(GlobalConsensus(RelayNetwork::get()), Parachain(1013));
		const BridgedNetwork: NetworkId =  Ethereum{ chain_id: 1 };
		BridgedLocation: MultiLocation = MultiLocation::new(0, (GlobalConsensus(BridgedNetwork::get()), AccountKey20 { network: None, key: GATEWAY }));
		BridgedLocationWithoutRegistry: MultiLocation = MultiLocation::new(0, GlobalConsensus(BridgedNetwork::get()));
		BridgedLocationWithoutGlobalConsensus: MultiLocation = MultiLocation::new(0, AccountKey20 { network: None, key: GATEWAY });
	}

	const GATEWAY: [u8; 20] = hex!("D184c103F7acc340847eEE82a0B909E3358bc28d");

	struct MockOkOutboundQueue;
	impl OutboundQueueTrait for MockOkOutboundQueue {
		type Ticket = ();
		type Balance = u128;

		fn validate(_: &Message) -> Result<((), Fees<Self::Balance>), SendError> {
			Ok(((), Fees { base: 1, delivery: 1 }))
		}

		fn submit(_: Self::Ticket) -> Result<MessageHash, SendError> {
			Ok(MessageHash::zero())
		}
	}
	struct MockErrOutboundQueue;
	impl OutboundQueueTrait for MockErrOutboundQueue {
		type Ticket = ();
		type Balance = u128;

		fn validate(_: &Message) -> Result<((), Fees<Self::Balance>), SendError> {
			Err(SendError::MessageTooLarge)
		}

		fn submit(_: Self::Ticket) -> Result<MessageHash, SendError> {
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

		let result =
			EthereumBlobExporter::<
				UniversalLocation,
				BridgedLocation,
				MockOkOutboundQueue,
				AgentIdOf,
			>::validate(network, channel, &mut universal_source, &mut destination, &mut message);
		assert_eq!(result, Err(XcmSendError::NotApplicable));
	}

	#[test]
	fn exporter_validate_with_invalid_destination_yields_missing_argument() {
		let network = BridgedNetwork::get();
		let channel: u32 = 0;
		let mut universal_source: Option<InteriorMultiLocation> = None;
		let mut destination: Option<InteriorMultiLocation> = None;
		let mut message: Option<Xcm<()>> = None;

		let result =
			EthereumBlobExporter::<
				UniversalLocation,
				BridgedLocation,
				MockOkOutboundQueue,
				AgentIdOf,
			>::validate(network, channel, &mut universal_source, &mut destination, &mut message);
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

		let result =
			EthereumBlobExporter::<
				UniversalLocation,
				BridgedLocation,
				MockOkOutboundQueue,
				AgentIdOf,
			>::validate(network, channel, &mut universal_source, &mut destination, &mut message);
		assert_eq!(result, Err(XcmSendError::NotApplicable));
	}

	#[test]
	fn exporter_validate_without_universal_source_yields_missing_argument() {
		let network = BridgedNetwork::get();
		let channel: u32 = 0;
		let mut universal_source: Option<InteriorMultiLocation> = None;
		let mut destination: Option<InteriorMultiLocation> = Here.into();
		let mut message: Option<Xcm<()>> = None;

		let result =
			EthereumBlobExporter::<
				UniversalLocation,
				BridgedLocation,
				MockOkOutboundQueue,
				AgentIdOf,
			>::validate(network, channel, &mut universal_source, &mut destination, &mut message);
		assert_eq!(result, Err(XcmSendError::MissingArgument));
	}

	#[test]
	fn exporter_validate_without_global_universal_location_yields_unroutable() {
		let network = BridgedNetwork::get();
		let channel: u32 = 0;
		let mut universal_source: Option<InteriorMultiLocation> = Here.into();
		let mut destination: Option<InteriorMultiLocation> = Here.into();
		let mut message: Option<Xcm<()>> = None;

		let result =
			EthereumBlobExporter::<
				UniversalLocation,
				BridgedLocation,
				MockOkOutboundQueue,
				AgentIdOf,
			>::validate(network, channel, &mut universal_source, &mut destination, &mut message);
		assert_eq!(result, Err(XcmSendError::Unroutable));
	}

	#[test]
	fn exporter_validate_without_global_bridge_location_yields_not_applicable() {
		let network = BridgedNetwork::get();
		let channel: u32 = 0;
		let mut universal_source: Option<InteriorMultiLocation> = Here.into();
		let mut destination: Option<InteriorMultiLocation> = Here.into();
		let mut message: Option<Xcm<()>> = None;

		let result =
			EthereumBlobExporter::<
				UniversalLocation,
				BridgedLocationWithoutGlobalConsensus,
				MockOkOutboundQueue,
				AgentIdOf,
			>::validate(network, channel, &mut universal_source, &mut destination, &mut message);
		assert_eq!(result, Err(XcmSendError::NotApplicable));
	}

	#[test]
	fn exporter_validate_without_registry_bridge_location_yields_not_applicable() {
		let network = BridgedNetwork::get();
		let channel: u32 = 0;
		let mut universal_source: Option<InteriorMultiLocation> = Here.into();
		let mut destination: Option<InteriorMultiLocation> = Here.into();
		let mut message: Option<Xcm<()>> = None;

		let result =
			EthereumBlobExporter::<
				UniversalLocation,
				BridgedLocationWithoutRegistry,
				MockOkOutboundQueue,
				AgentIdOf,
			>::validate(network, channel, &mut universal_source, &mut destination, &mut message);
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

		let result =
			EthereumBlobExporter::<
				UniversalLocation,
				BridgedLocation,
				MockOkOutboundQueue,
				AgentIdOf,
			>::validate(network, channel, &mut universal_source, &mut destination, &mut message);
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

		let result =
			EthereumBlobExporter::<
				UniversalLocation,
				BridgedLocation,
				MockOkOutboundQueue,
				AgentIdOf,
			>::validate(network, channel, &mut universal_source, &mut destination, &mut message);
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

		let result =
			EthereumBlobExporter::<
				UniversalLocation,
				BridgedLocation,
				MockOkOutboundQueue,
				AgentIdOf,
			>::validate(network, channel, &mut universal_source, &mut destination, &mut message);
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

		let result =
			EthereumBlobExporter::<
				UniversalLocation,
				BridgedLocation,
				MockOkOutboundQueue,
				AgentIdOf,
			>::validate(network, channel, &mut universal_source, &mut destination, &mut message);
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
			id: Concrete(
				X2(
					AccountKey20 { network: None, key: GATEWAY },
					AccountKey20 { network: None, key: token_address },
				)
				.into(),
			),
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

		let result =
			EthereumBlobExporter::<
				UniversalLocation,
				BridgedLocation,
				MockOkOutboundQueue,
				AgentIdOf,
			>::validate(network, channel, &mut universal_source, &mut destination, &mut message);

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

		let result =
			EthereumBlobExporter::<
				UniversalLocation,
				BridgedLocation,
				MockOkOutboundQueue,
				AgentIdOf,
			>::validate(network, channel, &mut universal_source, &mut destination, &mut message);

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
			id: Concrete(
				X2(
					AccountKey20 { network: None, key: GATEWAY },
					AccountKey20 { network: None, key: token_address },
				)
				.into(),
			),
			fun: Fungible(1000),
		}]
		.into();
		let filter: MultiAssetFilter = assets.clone().into();

		let mut message: Option<Xcm<()>> = Some(
			vec![
				UnpaidExecution { weight_limit: Unlimited, check_origin: None },
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

		let result =
			EthereumBlobExporter::<
				UniversalLocation,
				BridgedLocation,
				MockOkOutboundQueue,
				AgentIdOf,
			>::validate(network, channel, &mut universal_source, &mut destination, &mut message);

		assert!(result.is_ok());
	}

	#[test]
	fn exporter_deliver_with_submit_failure_yields_unroutable() {
		let result = EthereumBlobExporter::<
			UniversalLocation,
			BridgedLocation,
			MockErrOutboundQueue,
			AgentIdOf,
		>::deliver(hex!("deadbeef").to_vec());
		assert_eq!(result, Err(XcmSendError::Transport("other transport error")))
	}

	#[test]
	fn xcm_converter_convert_success_with_max_target_fee() {
		let network = BridgedNetwork::get();

		let fee = MultiAsset { id: Concrete(Here.into()), fun: Fungible(1000) };
		let fees: MultiAssets = vec![fee.clone()].into();

		let token_address: [u8; 20] = hex!("1000000000000000000000000000000000000000");
		let beneficiary_address: [u8; 20] = hex!("2000000000000000000000000000000000000000");

		let assets: MultiAssets = vec![MultiAsset {
			id: Concrete(
				X2(
					AccountKey20 { network: None, key: GATEWAY },
					AccountKey20 { network: None, key: token_address },
				)
				.into(),
			),
			fun: Fungible(1000),
		}]
		.into();
		let filter: MultiAssetFilter = assets.clone().into();

		let message: Xcm<()> = vec![
			WithdrawAsset(fees),
			BuyExecution { fees: fee.clone(), weight_limit: Unlimited },
			WithdrawAsset(assets),
			DepositAsset {
				assets: filter,
				beneficiary: X1(AccountKey20 { network: None, key: beneficiary_address }).into(),
			},
			SetTopic([0; 32]),
		]
		.into();
		let mut converter = XcmConverter::new(&message, &network, &GATEWAY);
		let expected_payload = AgentExecuteCommand::TransferToken {
			token: token_address.into(),
			recipient: beneficiary_address.into(),
			amount: 1000,
		};
		let result = converter.convert();
		assert_eq!(result, Ok((expected_payload, Some(&fee))));
	}

	#[test]
	fn xcm_converter_convert_success_without_max_target_fee() {
		let network = BridgedNetwork::get();

		let token_address: [u8; 20] = hex!("1000000000000000000000000000000000000000");
		let beneficiary_address: [u8; 20] = hex!("2000000000000000000000000000000000000000");

		let assets: MultiAssets = vec![MultiAsset {
			id: Concrete(
				X2(
					AccountKey20 { network: None, key: GATEWAY },
					AccountKey20 { network: None, key: token_address },
				)
				.into(),
			),
			fun: Fungible(1000),
		}]
		.into();
		let filter: MultiAssetFilter = assets.clone().into();

		let message: Xcm<()> = vec![
			UnpaidExecution { weight_limit: Unlimited, check_origin: None },
			WithdrawAsset(assets),
			DepositAsset {
				assets: filter,
				beneficiary: X1(AccountKey20 { network: None, key: beneficiary_address }).into(),
			},
			SetTopic([0; 32]),
		]
		.into();
		let mut converter = XcmConverter::new(&message, &network, &GATEWAY);
		let expected_payload = AgentExecuteCommand::TransferToken {
			token: token_address.into(),
			recipient: beneficiary_address.into(),
			amount: 1000,
		};
		let result = converter.convert();
		assert_eq!(result, Ok((expected_payload, None)));
	}

	#[test]
	fn xcm_converter_convert_with_wildcard_all_asset_filter_succeeds() {
		let network = BridgedNetwork::get();

		let token_address: [u8; 20] = hex!("1000000000000000000000000000000000000000");
		let beneficiary_address: [u8; 20] = hex!("2000000000000000000000000000000000000000");

		let assets: MultiAssets = vec![MultiAsset {
			id: Concrete(
				X2(
					AccountKey20 { network: None, key: GATEWAY },
					AccountKey20 { network: None, key: token_address },
				)
				.into(),
			),
			fun: Fungible(1000),
		}]
		.into();
		let filter: MultiAssetFilter = Wild(All);

		let message: Xcm<()> = vec![
			UnpaidExecution { weight_limit: Unlimited, check_origin: None },
			WithdrawAsset(assets),
			DepositAsset {
				assets: filter,
				beneficiary: X1(AccountKey20 { network: None, key: beneficiary_address }).into(),
			},
			SetTopic([0; 32]),
		]
		.into();
		let mut converter = XcmConverter::new(&message, &network, &GATEWAY);
		let expected_payload = AgentExecuteCommand::TransferToken {
			token: token_address.into(),
			recipient: beneficiary_address.into(),
			amount: 1000,
		};
		let result = converter.convert();
		assert_eq!(result, Ok((expected_payload, None)));
	}

	#[test]
	fn xcm_converter_convert_with_partial_message_yields_unexpected_end_of_xcm() {
		let network = BridgedNetwork::get();

		let token_address: [u8; 20] = hex!("1000000000000000000000000000000000000000");
		let assets: MultiAssets = vec![MultiAsset {
			id: Concrete(
				X2(
					AccountKey20 { network: None, key: GATEWAY },
					AccountKey20 { network: None, key: token_address },
				)
				.into(),
			),
			fun: Fungible(1000),
		}]
		.into();
		let message: Xcm<()> = vec![
			UnpaidExecution { weight_limit: Unlimited, check_origin: None },
			WithdrawAsset(assets),
		]
		.into();

		let mut converter = XcmConverter::new(&message, &network, &GATEWAY);
		let result = converter.convert();
		assert_eq!(result.err(), Some(XcmConverterError::UnexpectedEndOfXcm));
	}

	#[test]
	fn xcm_converter_convert_with_empty_xcm_yields_unexpected_end_of_xcm() {
		let network = BridgedNetwork::get();

		let message: Xcm<()> = vec![].into();

		let mut converter = XcmConverter::new(&message, &network, &GATEWAY);

		let result = converter.convert();
		assert_eq!(result.err(), Some(XcmConverterError::UnexpectedEndOfXcm));
	}

	#[test]
	fn xcm_converter_convert_without_max_target_fee_yields_target_fee_expected() {
		let network = BridgedNetwork::get();

		let token_address: [u8; 20] = hex!("1000000000000000000000000000000000000000");
		let beneficiary_address: [u8; 20] = hex!("2000000000000000000000000000000000000000");

		let assets: MultiAssets = vec![MultiAsset {
			id: Concrete(
				X2(
					AccountKey20 { network: None, key: GATEWAY },
					AccountKey20 { network: None, key: token_address },
				)
				.into(),
			),
			fun: Fungible(1000),
		}]
		.into();
		let filter: MultiAssetFilter = assets.clone().into();

		let message: Xcm<()> = vec![
			ClearTopic,
			WithdrawAsset(assets),
			DepositAsset {
				assets: filter,
				beneficiary: X1(AccountKey20 { network: None, key: beneficiary_address }).into(),
			},
			SetTopic([0; 32]),
		]
		.into();
		let mut converter = XcmConverter::new(&message, &network, &GATEWAY);

		let result = converter.convert();
		assert_eq!(result.err(), Some(XcmConverterError::TargetFeeExpected));
	}

	#[test]
	fn xcm_converter_convert_without_set_topic_suffix_yields_set_topic_expected() {
		let network = BridgedNetwork::get();

		let token_address: [u8; 20] = hex!("1000000000000000000000000000000000000000");
		let beneficiary_address: [u8; 20] = hex!("2000000000000000000000000000000000000000");

		let fee = MultiAsset { id: Concrete(Here.into()), fun: Fungible(1000) };
		let fees: MultiAssets = vec![fee.clone()].into();

		let assets: MultiAssets = vec![MultiAsset {
			id: Concrete(
				X2(
					AccountKey20 { network: None, key: GATEWAY },
					AccountKey20 { network: None, key: token_address },
				)
				.into(),
			),
			fun: Fungible(1000),
		}]
		.into();
		let filter: MultiAssetFilter = assets.clone().into();

		let message: Xcm<()> = vec![
			WithdrawAsset(fees),
			BuyExecution { fees: fee.clone(), weight_limit: Unlimited },
			WithdrawAsset(assets),
			DepositAsset {
				assets: filter,
				beneficiary: X1(AccountKey20 { network: None, key: beneficiary_address }).into(),
			},
			ClearTopic,
		]
		.into();
		let mut converter = XcmConverter::new(&message, &network, &GATEWAY);

		let result = converter.convert();
		assert_eq!(result.err(), Some(XcmConverterError::SetTopicExpected));
	}

	#[test]
	fn xcm_converter_convert_with_extra_instructions_yields_end_of_xcm_message_expected() {
		let network = BridgedNetwork::get();

		let token_address: [u8; 20] = hex!("1000000000000000000000000000000000000000");
		let beneficiary_address: [u8; 20] = hex!("2000000000000000000000000000000000000000");

		let fee = MultiAsset { id: Concrete(Here.into()), fun: Fungible(1000) };
		let fees: MultiAssets = vec![fee.clone()].into();

		let assets: MultiAssets = vec![MultiAsset {
			id: Concrete(
				X2(
					AccountKey20 { network: None, key: GATEWAY },
					AccountKey20 { network: None, key: token_address },
				)
				.into(),
			),
			fun: Fungible(1000),
		}]
		.into();
		let filter: MultiAssetFilter = assets.clone().into();

		let message: Xcm<()> = vec![
			WithdrawAsset(fees),
			BuyExecution { fees: fee.clone(), weight_limit: Unlimited },
			WithdrawAsset(assets),
			DepositAsset {
				assets: filter,
				beneficiary: X1(AccountKey20 { network: None, key: beneficiary_address }).into(),
			},
			SetTopic([0; 32]),
			ClearOrigin,
		]
		.into();
		let mut converter = XcmConverter::new(&message, &network, &GATEWAY);

		let result = converter.convert();
		assert_eq!(result.err(), Some(XcmConverterError::EndOfXcmMessageExpected));
	}

	#[test]
	fn xcm_converter_convert_without_withdraw_asset_yields_withdraw_expected() {
		let network = BridgedNetwork::get();

		let token_address: [u8; 20] = hex!("1000000000000000000000000000000000000000");
		let beneficiary_address: [u8; 20] = hex!("2000000000000000000000000000000000000000");

		let fee = MultiAsset { id: Concrete(Here.into()), fun: Fungible(1000) };
		let fees: MultiAssets = vec![fee.clone()].into();

		let assets: MultiAssets = vec![MultiAsset {
			id: Concrete(
				X2(
					AccountKey20 { network: None, key: GATEWAY },
					AccountKey20 { network: None, key: token_address },
				)
				.into(),
			),
			fun: Fungible(1000),
		}]
		.into();
		let filter: MultiAssetFilter = assets.clone().into();

		let message: Xcm<()> = vec![
			WithdrawAsset(fees),
			BuyExecution { fees: fee.clone(), weight_limit: Unlimited },
			DepositAsset {
				assets: filter,
				beneficiary: X1(AccountKey20 { network: None, key: beneficiary_address }).into(),
			},
			SetTopic([0; 32]),
		]
		.into();
		let mut converter = XcmConverter::new(&message, &network, &GATEWAY);

		let result = converter.convert();
		assert_eq!(result.err(), Some(XcmConverterError::WithdrawExpected));
	}

	#[test]
	fn xcm_converter_convert_without_withdraw_asset_yields_deposit_expected() {
		let network = BridgedNetwork::get();

		let token_address: [u8; 20] = hex!("1000000000000000000000000000000000000000");

		let fee = MultiAsset { id: Concrete(Here.into()), fun: Fungible(1000) };
		let fees: MultiAssets = vec![fee.clone()].into();

		let assets: MultiAssets = vec![MultiAsset {
			id: Concrete(
				X2(
					AccountKey20 { network: None, key: GATEWAY },
					AccountKey20 { network: None, key: token_address },
				)
				.into(),
			),
			fun: Fungible(1000),
		}]
		.into();

		let message: Xcm<()> = vec![
			WithdrawAsset(fees),
			BuyExecution { fees: fee.clone(), weight_limit: Unlimited },
			WithdrawAsset(assets),
			SetTopic([0; 32]),
		]
		.into();
		let mut converter = XcmConverter::new(&message, &network, &GATEWAY);

		let result = converter.convert();
		assert_eq!(result.err(), Some(XcmConverterError::DepositExpected));
	}

	#[test]
	fn xcm_converter_convert_without_assets_yields_no_reserve_assets() {
		let network = BridgedNetwork::get();

		let beneficiary_address: [u8; 20] = hex!("2000000000000000000000000000000000000000");

		let fee = MultiAsset { id: Concrete(Here.into()), fun: Fungible(1000) };
		let fees: MultiAssets = vec![fee.clone()].into();

		let assets: MultiAssets = vec![].into();
		let filter: MultiAssetFilter = assets.clone().into();

		let message: Xcm<()> = vec![
			WithdrawAsset(fees),
			BuyExecution { fees: fee.clone(), weight_limit: Unlimited },
			WithdrawAsset(assets),
			DepositAsset {
				assets: filter,
				beneficiary: X1(AccountKey20 { network: None, key: beneficiary_address }).into(),
			},
			SetTopic([0; 32]),
		]
		.into();
		let mut converter = XcmConverter::new(&message, &network, &GATEWAY);

		let result = converter.convert();
		assert_eq!(result.err(), Some(XcmConverterError::NoReserveAssets));
	}

	#[test]
	fn xcm_converter_convert_with_two_assets_yields_too_many_assets() {
		let network = BridgedNetwork::get();

		let token_address_1: [u8; 20] = hex!("1000000000000000000000000000000000000000");
		let token_address_2: [u8; 20] = hex!("1100000000000000000000000000000000000000");
		let beneficiary_address: [u8; 20] = hex!("2000000000000000000000000000000000000000");

		let fee = MultiAsset { id: Concrete(Here.into()), fun: Fungible(1000) };
		let fees: MultiAssets = vec![fee.clone()].into();

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
			WithdrawAsset(fees),
			BuyExecution { fees: fee.clone(), weight_limit: Unlimited },
			WithdrawAsset(assets),
			DepositAsset {
				assets: filter,
				beneficiary: X1(AccountKey20 { network: None, key: beneficiary_address }).into(),
			},
			SetTopic([0; 32]),
		]
		.into();
		let mut converter = XcmConverter::new(&message, &network, &GATEWAY);

		let result = converter.convert();
		assert_eq!(result.err(), Some(XcmConverterError::TooManyAssets));
	}

	#[test]
	fn xcm_converter_convert_without_consuming_filter_yields_filter_does_not_consume_all_assets() {
		let network = BridgedNetwork::get();

		let token_address: [u8; 20] = hex!("1000000000000000000000000000000000000000");
		let beneficiary_address: [u8; 20] = hex!("2000000000000000000000000000000000000000");

		let fee = MultiAsset { id: Concrete(Here.into()), fun: Fungible(1000) };
		let fees: MultiAssets = vec![fee.clone()].into();

		let assets: MultiAssets = vec![MultiAsset {
			id: Concrete(
				X2(
					AccountKey20 { network: None, key: GATEWAY },
					AccountKey20 { network: None, key: token_address },
				)
				.into(),
			),
			fun: Fungible(1000),
		}]
		.into();
		let filter: MultiAssetFilter = Wild(WildMultiAsset::AllCounted(0));

		let message: Xcm<()> = vec![
			WithdrawAsset(fees),
			BuyExecution { fees: fee.clone(), weight_limit: Unlimited },
			WithdrawAsset(assets),
			DepositAsset {
				assets: filter,
				beneficiary: X1(AccountKey20 { network: None, key: beneficiary_address }).into(),
			},
			SetTopic([0; 32]),
		]
		.into();
		let mut converter = XcmConverter::new(&message, &network, &GATEWAY);

		let result = converter.convert();
		assert_eq!(result.err(), Some(XcmConverterError::FilterDoesNotConsumeAllAssets));
	}

	#[test]
	fn xcm_converter_convert_with_non_fungible_asset_yields_asset_not_concrete_fungible() {
		let network = BridgedNetwork::get();

		let token_address: [u8; 20] = hex!("1000000000000000000000000000000000000000");
		let beneficiary_address: [u8; 20] = hex!("2000000000000000000000000000000000000000");

		let fee = MultiAsset { id: Concrete(Here.into()), fun: Fungible(1000) };
		let fees: MultiAssets = vec![fee.clone()].into();

		let assets: MultiAssets = vec![MultiAsset {
			id: Concrete(
				X2(
					AccountKey20 { network: None, key: GATEWAY },
					AccountKey20 { network: None, key: token_address },
				)
				.into(),
			),
			fun: NonFungible(AssetInstance::Index(0)),
		}]
		.into();
		let filter: MultiAssetFilter = Wild(WildMultiAsset::AllCounted(1));

		let message: Xcm<()> = vec![
			WithdrawAsset(fees),
			BuyExecution { fees: fee.clone(), weight_limit: Unlimited },
			WithdrawAsset(assets),
			DepositAsset {
				assets: filter,
				beneficiary: X1(AccountKey20 { network: None, key: beneficiary_address }).into(),
			},
			SetTopic([0; 32]),
		]
		.into();
		let mut converter = XcmConverter::new(&message, &network, &GATEWAY);

		let result = converter.convert();
		assert_eq!(result.err(), Some(XcmConverterError::AssetNotConcreteFungible));
	}

	#[test]
	fn xcm_converter_convert_with_zero_amount_asset_yields_zero_asset_transfer() {
		let network = BridgedNetwork::get();

		let token_address: [u8; 20] = hex!("1000000000000000000000000000000000000000");
		let beneficiary_address: [u8; 20] = hex!("2000000000000000000000000000000000000000");

		let fee = MultiAsset { id: Concrete(Here.into()), fun: Fungible(1000) };
		let fees: MultiAssets = vec![fee.clone()].into();

		let assets: MultiAssets = vec![MultiAsset {
			id: Concrete(
				X2(
					AccountKey20 { network: None, key: GATEWAY },
					AccountKey20 { network: None, key: token_address },
				)
				.into(),
			),
			fun: Fungible(0),
		}]
		.into();
		let filter: MultiAssetFilter = Wild(WildMultiAsset::AllCounted(1));

		let message: Xcm<()> = vec![
			WithdrawAsset(fees),
			BuyExecution { fees: fee.clone(), weight_limit: Unlimited },
			WithdrawAsset(assets),
			DepositAsset {
				assets: filter,
				beneficiary: X1(AccountKey20 { network: None, key: beneficiary_address }).into(),
			},
			SetTopic([0; 32]),
		]
		.into();
		let mut converter = XcmConverter::new(&message, &network, &GATEWAY);

		let result = converter.convert();
		assert_eq!(result.err(), Some(XcmConverterError::ZeroAssetTransfer));
	}

	#[test]
	fn xcm_converter_convert_non_ethereum_asset_yields_asset_resolution_failed() {
		let network = BridgedNetwork::get();

		let beneficiary_address: [u8; 20] = hex!("2000000000000000000000000000000000000000");

		let fee = MultiAsset { id: Concrete(Here.into()), fun: Fungible(1000) };
		let fees: MultiAssets = vec![fee.clone()].into();

		let assets: MultiAssets = vec![MultiAsset {
			id: Concrete(X3(GlobalConsensus(Polkadot), Parachain(1000), GeneralIndex(0)).into()),
			fun: Fungible(1000),
		}]
		.into();
		let filter: MultiAssetFilter = Wild(WildMultiAsset::AllCounted(1));

		let message: Xcm<()> = vec![
			WithdrawAsset(fees),
			BuyExecution { fees: fee.clone(), weight_limit: Unlimited },
			WithdrawAsset(assets),
			DepositAsset {
				assets: filter,
				beneficiary: X1(AccountKey20 { network: None, key: beneficiary_address }).into(),
			},
			SetTopic([0; 32]),
		]
		.into();
		let mut converter = XcmConverter::new(&message, &network, &GATEWAY);

		let result = converter.convert();
		assert_eq!(result.err(), Some(XcmConverterError::AssetResolutionFailed));
	}

	#[test]
	fn xcm_converter_convert_non_ethereum_chain_asset_yields_asset_resolution_failed() {
		let network = BridgedNetwork::get();

		let token_address: [u8; 20] = hex!("1000000000000000000000000000000000000000");
		let beneficiary_address: [u8; 20] = hex!("2000000000000000000000000000000000000000");

		let fee = MultiAsset { id: Concrete(Here.into()), fun: Fungible(1000) };
		let fees: MultiAssets = vec![fee.clone()].into();

		let assets: MultiAssets = vec![MultiAsset {
			id: Concrete(
				X2(
					AccountKey20 { network: Some(network), key: GATEWAY },
					AccountKey20 { network: Some(Ethereum { chain_id: 2 }), key: token_address },
				)
				.into(),
			),
			fun: Fungible(1000),
		}]
		.into();
		let filter: MultiAssetFilter = Wild(WildMultiAsset::AllCounted(1));

		let message: Xcm<()> = vec![
			WithdrawAsset(fees),
			BuyExecution { fees: fee.clone(), weight_limit: Unlimited },
			WithdrawAsset(assets),
			DepositAsset {
				assets: filter,
				beneficiary: X1(AccountKey20 { network: None, key: beneficiary_address }).into(),
			},
			SetTopic([0; 32]),
		]
		.into();
		let mut converter = XcmConverter::new(&message, &network, &GATEWAY);

		let result = converter.convert();
		assert_eq!(result.err(), Some(XcmConverterError::AssetResolutionFailed));
	}

	#[test]
	fn xcm_converter_convert_bad_registry_yields_asset_resolution_failed() {
		let network = BridgedNetwork::get();

		let token_address: [u8; 20] = hex!("1000000000000000000000000000000000000000");
		let beneficiary_address: [u8; 20] = hex!("2000000000000000000000000000000000000000");

		const BAD_REGISTRY: [u8; 20] = hex!("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF");

		let fee = MultiAsset { id: Concrete(Here.into()), fun: Fungible(1000) };
		let fees: MultiAssets = vec![fee.clone()].into();

		let assets: MultiAssets = vec![MultiAsset {
			id: Concrete(
				X2(
					AccountKey20 { network: Some(network), key: BAD_REGISTRY },
					AccountKey20 { network: Some(network), key: token_address },
				)
				.into(),
			),
			fun: Fungible(1000),
		}]
		.into();
		let filter: MultiAssetFilter = Wild(WildMultiAsset::AllCounted(1));

		let message: Xcm<()> = vec![
			WithdrawAsset(fees),
			BuyExecution { fees: fee.clone(), weight_limit: Unlimited },
			WithdrawAsset(assets),
			DepositAsset {
				assets: filter,
				beneficiary: X1(AccountKey20 { network: None, key: beneficiary_address }).into(),
			},
			SetTopic([0; 32]),
		]
		.into();
		let mut converter = XcmConverter::new(&message, &network, &GATEWAY);

		let result = converter.convert();
		assert_eq!(result.err(), Some(XcmConverterError::AssetResolutionFailed));
	}

	#[test]
	fn xcm_converter_convert_non_ethereum_chain_registry_asset_yields_asset_resolution_failed() {
		let network = BridgedNetwork::get();

		let token_address: [u8; 20] = hex!("1000000000000000000000000000000000000000");
		let beneficiary_address: [u8; 20] = hex!("2000000000000000000000000000000000000000");

		let fee = MultiAsset { id: Concrete(Here.into()), fun: Fungible(1000) };
		let fees: MultiAssets = vec![fee.clone()].into();

		let assets: MultiAssets = vec![MultiAsset {
			id: Concrete(
				X2(
					AccountKey20 { network: Some(Ethereum { chain_id: 2 }), key: GATEWAY },
					AccountKey20 { network: Some(network), key: token_address },
				)
				.into(),
			),
			fun: Fungible(1000),
		}]
		.into();
		let filter: MultiAssetFilter = Wild(WildMultiAsset::AllCounted(1));

		let message: Xcm<()> = vec![
			WithdrawAsset(fees),
			BuyExecution { fees: fee.clone(), weight_limit: Unlimited },
			WithdrawAsset(assets),
			DepositAsset {
				assets: filter,
				beneficiary: X1(AccountKey20 { network: None, key: beneficiary_address }).into(),
			},
			SetTopic([0; 32]),
		]
		.into();
		let mut converter = XcmConverter::new(&message, &network, &GATEWAY);

		let result = converter.convert();
		assert_eq!(result.err(), Some(XcmConverterError::AssetResolutionFailed));
	}

	#[test]
	fn xcm_converter_convert_with_non_ethereum_beneficiary_yields_beneficiary_resolution_failed() {
		let network = BridgedNetwork::get();

		let token_address: [u8; 20] = hex!("1000000000000000000000000000000000000000");

		let beneficiary_address: [u8; 32] =
			hex!("2000000000000000000000000000000000000000000000000000000000000000");

		let fee = MultiAsset { id: Concrete(Here.into()), fun: Fungible(1000) };
		let fees: MultiAssets = vec![fee.clone()].into();

		let assets: MultiAssets = vec![MultiAsset {
			id: Concrete(
				X2(
					AccountKey20 { network: None, key: GATEWAY },
					AccountKey20 { network: None, key: token_address },
				)
				.into(),
			),
			fun: Fungible(1000),
		}]
		.into();
		let filter: MultiAssetFilter = Wild(WildMultiAsset::AllCounted(1));

		let message: Xcm<()> = vec![
			WithdrawAsset(fees),
			BuyExecution { fees: fee.clone(), weight_limit: Unlimited },
			WithdrawAsset(assets),
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
		let mut converter = XcmConverter::new(&message, &network, &GATEWAY);

		let result = converter.convert();
		assert_eq!(result.err(), Some(XcmConverterError::BeneficiaryResolutionFailed));
	}

	#[test]
	fn xcm_converter_convert_with_non_ethereum_chain_beneficiary_yields_beneficiary_resolution_failed(
	) {
		let network = BridgedNetwork::get();

		let token_address: [u8; 20] = hex!("1000000000000000000000000000000000000000");
		let beneficiary_address: [u8; 20] = hex!("2000000000000000000000000000000000000000");

		let fee = MultiAsset { id: Concrete(Here.into()), fun: Fungible(1000) };
		let fees: MultiAssets = vec![fee.clone()].into();

		let assets: MultiAssets = vec![MultiAsset {
			id: Concrete(
				X2(
					AccountKey20 { network: None, key: GATEWAY },
					AccountKey20 { network: None, key: token_address },
				)
				.into(),
			),
			fun: Fungible(1000),
		}]
		.into();
		let filter: MultiAssetFilter = Wild(WildMultiAsset::AllCounted(1));

		let message: Xcm<()> = vec![
			WithdrawAsset(fees),
			BuyExecution { fees: fee.clone(), weight_limit: Unlimited },
			WithdrawAsset(assets),
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
		let mut converter = XcmConverter::new(&message, &network, &GATEWAY);

		let result = converter.convert();
		assert_eq!(result.err(), Some(XcmConverterError::BeneficiaryResolutionFailed));
	}
}
