use crate::XcmExportFeeToSibling;
use frame_support::{parameter_types, sp_runtime::testing::H256};
use snowbridge_core::outbound::{Fee, Message, SendError, SendMessage, SendMessageFeeProvider};
use xcm::prelude::{
	Here, MultiAsset, MultiAssets, MultiLocation, NetworkId, Parachain, Rococo, XcmContext,
	XcmError, XcmHash, XcmResult, X1,
};
use xcm_builder::HandleFee;
use xcm_executor::{
	traits::{FeeReason, TransactAsset},
	Assets,
};

parameter_types! {
	pub EthereumNetwork: NetworkId = NetworkId::Ethereum { chain_id: 11155111 };
	pub TokenLocation: MultiLocation = MultiLocation::parent();
}

struct MockOkOutboundQueue;
impl SendMessage for MockOkOutboundQueue {
	type Ticket = ();

	fn validate(_: &Message) -> Result<(Self::Ticket, Fee<Self::Balance>), SendError> {
		Ok(((), Fee { local: 1, remote: 1 }))
	}

	fn deliver(_: Self::Ticket) -> Result<H256, SendError> {
		Ok(H256::zero())
	}
}

impl SendMessageFeeProvider for MockOkOutboundQueue {
	type Balance = u128;

	fn local_fee() -> Self::Balance {
		1
	}
}
struct MockErrOutboundQueue;
impl SendMessage for MockErrOutboundQueue {
	type Ticket = ();

	fn validate(_: &Message) -> Result<(Self::Ticket, Fee<Self::Balance>), SendError> {
		Err(SendError::MessageTooLarge)
	}

	fn deliver(_: Self::Ticket) -> Result<H256, SendError> {
		Err(SendError::MessageTooLarge)
	}
}

impl SendMessageFeeProvider for MockErrOutboundQueue {
	type Balance = u128;

	fn local_fee() -> Self::Balance {
		1
	}
}

pub struct SuccessfulTransactor;
impl TransactAsset for SuccessfulTransactor {
	fn can_check_in(
		_origin: &MultiLocation,
		_what: &MultiAsset,
		_context: &XcmContext,
	) -> XcmResult {
		Ok(())
	}

	fn can_check_out(
		_dest: &MultiLocation,
		_what: &MultiAsset,
		_context: &XcmContext,
	) -> XcmResult {
		Ok(())
	}

	fn deposit_asset(
		_what: &MultiAsset,
		_who: &MultiLocation,
		_context: Option<&XcmContext>,
	) -> XcmResult {
		Ok(())
	}

	fn withdraw_asset(
		_what: &MultiAsset,
		_who: &MultiLocation,
		_context: Option<&XcmContext>,
	) -> Result<Assets, XcmError> {
		Ok(Assets::default())
	}

	fn internal_transfer_asset(
		_what: &MultiAsset,
		_from: &MultiLocation,
		_to: &MultiLocation,
		_context: &XcmContext,
	) -> Result<Assets, XcmError> {
		Ok(Assets::default())
	}
}

pub struct NotFoundTransactor;
impl TransactAsset for NotFoundTransactor {
	fn can_check_in(
		_origin: &MultiLocation,
		_what: &MultiAsset,
		_context: &XcmContext,
	) -> XcmResult {
		Err(XcmError::AssetNotFound)
	}

	fn can_check_out(
		_dest: &MultiLocation,
		_what: &MultiAsset,
		_context: &XcmContext,
	) -> XcmResult {
		Err(XcmError::AssetNotFound)
	}

	fn deposit_asset(
		_what: &MultiAsset,
		_who: &MultiLocation,
		_context: Option<&XcmContext>,
	) -> XcmResult {
		Err(XcmError::AssetNotFound)
	}

	fn withdraw_asset(
		_what: &MultiAsset,
		_who: &MultiLocation,
		_context: Option<&XcmContext>,
	) -> Result<Assets, XcmError> {
		Err(XcmError::AssetNotFound)
	}

	fn internal_transfer_asset(
		_what: &MultiAsset,
		_from: &MultiLocation,
		_to: &MultiLocation,
		_context: &XcmContext,
	) -> Result<Assets, XcmError> {
		Err(XcmError::AssetNotFound)
	}
}

#[test]
fn handle_fee_success() {
	let fee: MultiAssets = MultiAsset::from((MultiLocation::parent(), 10_u128)).into();
	let ctx = XcmContext {
		origin: Some(MultiLocation { parents: 1, interior: X1(Parachain(1000)) }),
		message_id: XcmHash::default(),
		topic: None,
	};
	let reason = FeeReason::Export { network: EthereumNetwork::get(), destination: Here };
	let result = XcmExportFeeToSibling::<
		u128,
		u64,
		TokenLocation,
		EthereumNetwork,
		SuccessfulTransactor,
		MockOkOutboundQueue,
	>::handle_fee(fee, Some(&ctx), reason)
	.unwrap();
	let local_fee =
		MultiAsset::from((MultiLocation::parent(), MockOkOutboundQueue::local_fee())).into();
	// assert only local fee left
	assert_eq!(result, local_fee)
}

#[test]
fn handle_fee_success_but_not_for_ethereum() {
	let fee: MultiAssets = MultiAsset::from((MultiLocation::parent(), 10_u128)).into();
	let ctx = XcmContext { origin: None, message_id: XcmHash::default(), topic: None };
	let reason = FeeReason::Export { network: Rococo, destination: Here };
	let result = XcmExportFeeToSibling::<
		u128,
		u64,
		TokenLocation,
		EthereumNetwork,
		SuccessfulTransactor,
		MockOkOutboundQueue,
	>::handle_fee(fee.clone(), Some(&ctx), reason)
	.unwrap();
	// assert fee not touched and just forward to the next handler
	assert_eq!(result, fee)
}
