// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
//! # Runtime Common
//!
//! Common traits and types shared by runtimes.
#![cfg_attr(not(feature = "std"), no_std)]

use bp_rococo::AccountId;
use core::marker::PhantomData;
use frame_support::traits::Get;
use snowbridge_core::{outbound::SendMessageFeeProvider, sibling_sovereign_account_raw};
use xcm::prelude::*;
use xcm_builder::{deposit_or_burn_fee, HandleFee};
use xcm_executor::traits::{FeeReason, TransactAsset};

/// A `HandleFee` implementation that takes fees from `ExportMessage` XCM instructions
/// to Snowbridge and splits off the local remote fee and deposits it to the origin
/// parachain sovereign account. The local fee is then returned back to be handled by
/// the next fee handler in the chain. Most likely the treasury account.
pub struct XcmExportFeeToSnowbridge<TokenLocation, EthereumNetwork, AssetTransactor, OutboundQueue>(
	PhantomData<(TokenLocation, EthereumNetwork, AssetTransactor, OutboundQueue)>,
);

impl<
		TokenLocation: Get<MultiLocation>,
		EthereumNetwork: Get<NetworkId>,
		AssetTransactor: TransactAsset,
		OutboundQueue: SendMessageFeeProvider<Balance = bp_rococo::Balance>,
	> HandleFee
	for XcmExportFeeToSnowbridge<TokenLocation, EthereumNetwork, AssetTransactor, OutboundQueue>
{
	fn handle_fee(
		fees: MultiAssets,
		context: Option<&XcmContext>,
		reason: FeeReason,
	) -> MultiAssets {
		let token_location = TokenLocation::get();

		// Check the reason to see if this export is for snowbridge.
		let snowbridge_export = matches!(
			reason,
			FeeReason::Export { network: bridged_network, destination }
				if bridged_network == EthereumNetwork::get() && destination == Here
		);

		// Get the parachain sovereign from the `context`.
		let maybe_para_sovereign = if let Some(XcmContext {
			origin: Some(MultiLocation { parents: 1, interior }),
			..
		}) = context
		{
			if let Some(Parachain(sibling_para_id)) = interior.first() {
				let account: AccountId =
					sibling_sovereign_account_raw((*sibling_para_id).into()).into();
				Some(account)
			} else {
				None
			}
		} else {
			None
		};

		// Get the total fee offered by export message.
		let maybe_total_supplied_fee = fees
			.inner()
			.iter()
			.enumerate()
			.filter_map(|(index, asset)| {
				if let MultiAsset { id: Concrete(location), fun: Fungible(amount) } = asset {
					if *location == token_location {
						return Some((index, amount))
					}
				}
				return None
			})
			.next();

		if let (true, Some(para_sovereign), Some((fee_index, total_fee))) =
			(snowbridge_export, maybe_para_sovereign, maybe_total_supplied_fee)
		{
			let remote_fee = total_fee.saturating_sub(OutboundQueue::local_fee());
			if remote_fee > 0 {
				// Send remote fee to origin
				deposit_or_burn_fee::<AssetTransactor, _>(
					MultiAsset { id: Concrete(token_location), fun: Fungible(remote_fee) }.into(),
					context,
					para_sovereign,
				);
				// Return remaining fee to the next fee handler in the chain.
				let mut modified_fees = fees.inner().clone();
				modified_fees.remove(fee_index);
				modified_fees.push(MultiAsset {
					id: Concrete(token_location),
					fun: Fungible(total_fee - remote_fee),
				});
				return modified_fees.into()
			}
		}

		log::trace!(
			target: "xcm::fees",
			"XcmExportFeeToSnowbridge skipped: {fees:?}, context: {context:?}, reason: {reason:?}",
		);
		return fees
	}
}
