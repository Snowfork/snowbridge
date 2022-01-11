use frame_support::dispatch::DispatchResult;

pub trait XcmReserveTransfer<AccountId, Origin> {
	fn reserve_transfer(
		origin: Origin,
		asset_id: u128,
		para_id: u32,
		recipient: &AccountId,
		amount: u128,
	) -> DispatchResult;
}
