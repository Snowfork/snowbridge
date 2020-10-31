
use codec::FullCodec;
use sp_core::H160;
use sp_runtime::{
	traits::{CheckedConversion, Convert, MaybeSerializeDeserialize, SaturatedConversion},
	DispatchResult,
};
use sp_std::{
	cmp::{Eq, PartialEq},
	collections::btree_set::BTreeSet,
	convert::{TryFrom, TryInto},
	fmt::Debug,
	marker::PhantomData,
	prelude::*,
	result,
};

use xcm::v0::{
	Error, Junction,
	MultiAsset as XMultiAsset,
	MultiLocation as XMultiLocation,
	Result
};

use artemis_core::assets::MultiAsset;

use xcm_executor::traits::{FilterAssetLocation, LocationConversion, MatchesFungible, NativeAsset, TransactAsset};

use frame_support::{debug, traits::Get};


pub struct MultiAssetAdapter<PalletMultiAsset, Matcher, AccountIdConverter, AccountId, AssetId>(
	PhantomData<(
		PalletMultiAsset,
		Matcher,
		AccountIdConverter,
		AccountId,
		AssetId,
	)>,
);

impl<
		PalletMultiAsset: artemis_core::assets::MultiAsset<AccountId, AssetId = H160>,
		Matcher: MatchesFungible<MultiCurrency::Balance>,
		AccountIdConverter: LocationConversion<AccountId>,
		AccountId: sp_std::fmt::Debug,
		CurrencyId: FullCodec + Eq + PartialEq + Copy + MaybeSerializeDeserialize + Debug,
	> TransactAsset
	for MultiCurrencyAdapter<MultiCurrency, Matcher, AccountIdConverter, AccountId, CurrencyIdConverter, CurrencyId>
{
	fn deposit_asset(asset: &MultiAsset, location: &MultiLocation) -> Result {
		debug::info!("------------------------------------------------");
		debug::info!(">>> trying deposit. asset: {:?}, location: {:?}", asset, location);
		let who = AccountIdConverter::from_location(location).ok_or(())?;
		debug::info!("who: {:?}", who);
		let currency_id = CurrencyIdConverter::from_asset(asset).ok_or(())?;
		debug::info!("currency_id: {:?}", currency_id);
		let amount = Matcher::matches_fungible(&asset).ok_or(())?.saturated_into();
		debug::info!("amount: {:?}", amount);
		let balance_amount = amount.try_into().map_err(|_| ())?;
		debug::info!("balance amount: {:?}", balance_amount);
		MultiCurrency::deposit(currency_id, &who, balance_amount).map_err(|_| ())?;
		debug::info!(">>> success deposit.");
		debug::info!("------------------------------------------------");
		Ok(())
	}

	fn withdraw_asset(asset: &MultiAsset, location: &MultiLocation) -> result::Result<MultiAsset, Error> {
		debug::info!("------------------------------------------------");
		debug::info!(">>> trying withdraw. asset: {:?}, location: {:?}", asset, location);
		let who = AccountIdConverter::from_location(location).ok_or(())?;
		debug::info!("who: {:?}", who);
		let currency_id = CurrencyIdConverter::from_asset(asset).ok_or(())?;
		debug::info!("currency_id: {:?}", currency_id);
		let amount = Matcher::matches_fungible(&asset).ok_or(())?.saturated_into();
		debug::info!("amount: {:?}", amount);
		let balance_amount = amount.try_into().map_err(|_| ())?;
		debug::info!("balance amount: {:?}", balance_amount);
		MultiCurrency::withdraw(currency_id, &who, balance_amount).map_err(|_| ())?;
		debug::info!(">>> success withdraw.");
		debug::info!("------------------------------------------------");
		Ok(asset.clone())
	}
}
