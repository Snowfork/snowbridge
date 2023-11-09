use bp_rococo::AccountId;
use core::marker::PhantomData;
use frame_support::traits::Get;
use snowbridge_core::{outbound::SendMessageFeeProvider, sibling_sovereign_account_raw};
use xcm::prelude::*;
use xcm_builder::{deposit_or_burn_fee, HandleFee};
use xcm_executor::traits::{FeeReason, TransactAsset};

/// A `HandleFee` implementation that takes fees from `ExportMessage` XCM instructions
/// to Snowbridge and holds it in a receiver account. Burns the fees in case of a failure.
pub struct XcmExportFeeToSnowbridge<
	TokenLocation,
	EthereumNetwork,
	ReceiverAccount,
	AssetTransactor,
	OutboundQueue,
>(PhantomData<(TokenLocation, EthereumNetwork, ReceiverAccount, AssetTransactor, OutboundQueue)>);

impl<
		TokenLocation: Get<MultiLocation>,
		EthereumNetwork: Get<NetworkId>,
		ReceiverAccount: Get<AccountId>,
		AssetTransactor: TransactAsset,
		OutboundQueue: SendMessageFeeProvider<Balance = bp_rococo::Balance>,
	> HandleFee
	for XcmExportFeeToSnowbridge<
		TokenLocation,
		EthereumNetwork,
		ReceiverAccount,
		AssetTransactor,
		OutboundQueue,
	>
{
	fn handle_fee(
		fees: MultiAssets,
		context: Option<&XcmContext>,
		reason: FeeReason,
	) -> MultiAssets {
		let token_location = TokenLocation::get();
		let mut fees = fees.into_inner();

		if matches!(reason, FeeReason::Export { network: bridged_network, destination }
				if bridged_network == EthereumNetwork::get() && destination == Here)
		{
			log::info!(
				target: "xcm::fees",
				"XcmExportFeeToSnowbridge fees: {fees:?}, context: {context:?}, reason: {reason:?}",
			);

			let fee_item_index = fees.iter().position(|asset| {
				matches!(
					asset,
					MultiAsset { id: Concrete(location), fun: Fungible(..)}
						if *location == token_location,
				)
			});
			// Find the fee asset.
			let fee_item = if let Some(element) = fee_item_index {
				fees.remove(element)
			} else {
				return fees.into()
			};

			let receiver = ReceiverAccount::get();
			// There is an origin so split fee into parts.
			if let Some(XcmContext {
				origin: Some(MultiLocation { parents: 1, interior }), ..
			}) = context
			{
				if let Some(Parachain(sibling_para_id)) = interior.first() {
					let account: AccountId =
						sibling_sovereign_account_raw((*sibling_para_id).into()).into();
					let local_fee = OutboundQueue::local_fee();
					if let Fungible(amount) = fee_item.fun {
						let remote_fee = amount.saturating_sub(local_fee);

						// Send local fee to receiver
						deposit_or_burn_fee::<AssetTransactor, _>(
							MultiAsset {
								id: Concrete(token_location),
								fun: Fungible(amount - remote_fee),
							}
							.into(),
							context,
							receiver,
						);
						// Send remote fee to origin
						deposit_or_burn_fee::<AssetTransactor, _>(
							MultiAsset { id: Concrete(token_location), fun: Fungible(remote_fee) }
								.into(),
							context,
							account,
						);
					} else {
						// Push the fee item back and bail out to let other handlers run.
						fees.push(fee_item);
						return fees.into()
					}
				} else {
					// Origin conversion failed so send the full fee to the receiver.
					deposit_or_burn_fee::<AssetTransactor, _>(fee_item.into(), context, receiver);
				}
			} else {
				// There is no context so send the full fee to the receiver.
				deposit_or_burn_fee::<AssetTransactor, _>(fee_item.into(), context, receiver);
			}
		}

		fees.into()
	}
}
