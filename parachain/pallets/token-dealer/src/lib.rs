#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{decl_event, decl_error, decl_module, decl_storage,
	dispatch::DispatchResult,
	traits::Get, Parameter
};
use frame_system::ensure_signed;
use sp_runtime::{
	traits::{AtLeast32BitUnsigned, Convert, MaybeSerializeDeserialize, Member},
	RuntimeDebug,
};
use sp_std::convert::TryInto;
use sp_std::prelude::*;
use sp_std::vec;

use cumulus_primitives::{relay_chain::Balance as RelayChainBalance, ParaId};
use xcm::v0::{Junction, MultiAsset, MultiLocation, NetworkId, Order, Xcm, ExecuteXcm};
use xcm::VersionedXcm;

use xcm_executor::traits::LocationConversion;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

const ASSETS_PALLET_INDEX: u8 = 11;

#[derive(Encode, Decode, Eq, PartialEq, Clone, Copy, RuntimeDebug)]
pub enum AssetId {
	ETH,
	ERC20([u8; 20])
}

/// Identity of a cross-chain asset.
#[derive(Encode, Decode, Eq, PartialEq, Clone, RuntimeDebug)]
pub struct XAssetId {
	/// The reserve chain of the asset.
	pub reserve_chain: ParaId,
	/// The identity of the asset.
	pub asset: AssetId
}

impl Into<MultiLocation> for XAssetId {
	fn into(self) -> MultiLocation {
		match self.asset {
			AssetId::ETH =>
				MultiLocation::X2(
					Junction::PalletInstance { id: ASSETS_PALLET_INDEX },
					Junction::AccountKey20 { network: NetworkId::Any, key: [0; 20] }),
			AssetId::ERC20(key) =>
				MultiLocation::X2(
					Junction::PalletInstance { id: ASSETS_PALLET_INDEX },
					Junction::AccountKey20 { network: NetworkId::Any, key }),
			}
	}
}

pub trait Trait: frame_system::Trait {
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;

	type Balance: Parameter + Member + AtLeast32BitUnsigned + Default + Copy + MaybeSerializeDeserialize + Into<u128>;

	type ToRelayChainBalance: Convert<Self::Balance, RelayChainBalance>;

	type AccountIdConverter: LocationConversion<Self::AccountId>;

	type AccountId32Converter: Convert<Self::AccountId, [u8; 32]>;

	type RelayChainNetworkId: Get<NetworkId>;

	type ParaId: Get<ParaId>;

	type XcmExecutor: ExecuteXcm;
}

decl_storage! {
	trait Store for Module<T: Trait> as TokenDealer {}
}

decl_event! {
	pub enum Event<T> where
		<T as frame_system::Trait>::AccountId,
		<T as Trait>::Balance,
	{
		/// Transferred to relay chain. [src, dest, amount]
		TransferredToRelayChain(AccountId, AccountId, Balance),

		/// Transferred to parachain. [x_asset_id, src, para_id, dest, dest_network, amount]
		TransferredToParachain(XAssetId, AccountId, ParaId, AccountId, NetworkId, Balance),
	}
}

decl_error! {
	pub enum Error for Module<T: Trait> {
		/// Location given was invalid or unsupported.
		BadLocation,
		/// The XCM message version is not supported.
		BadVersion,
		/// XCM execution failed
		ExecutionFailed,
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		fn deposit_event() = default;

		/// Transfer DOT to relay chain.
		#[weight = 10]
		pub fn transfer_to_relaychain(origin, dest: T::AccountId, amount: T::Balance) -> DispatchResult {

			let who = ensure_signed(origin.clone())?;
			let xcm = Self::make_xcm_upward_transfer(&dest, amount);

			Self::execute(&who, xcm.into())?;

			Self::deposit_event(Event::<T>::TransferredToRelayChain(who, dest, amount));

			Ok(())
		}

		/// Transfer bridged ethereum assets to a sibling parachain.
		#[weight = 10]
		pub fn transfer_to_parachain(
			origin,
			asset: XAssetId,
			para_id: ParaId,
			network: NetworkId,
			account: T::AccountId,
			amount: T::Balance
		) -> DispatchResult {
			let who = ensure_signed(origin.clone())?;

			if para_id == T::ParaId::get() {
				return Ok(());
			}

			let location: MultiLocation = asset.clone().into();

			let xcm = if asset.reserve_chain == T::ParaId::get() {
				Self::make_xcm_lateral_transfer_native(
					location,
					para_id,
					&network,
					&account,
					amount)
			} else {
				Self::make_xcm_lateral_transfer_foreign(
					asset.reserve_chain,
					location,
					para_id,
					&network,
					&account,
					amount)
			};

			Self::execute(&who, xcm.into())?;

			Self::deposit_event(
				Event::<T>::TransferredToParachain(asset, who, para_id, account, network, amount),
			);

			Ok(())
		}
	}
}

impl<T: Trait> Module<T> {

	fn execute(who: &T::AccountId, xcm: VersionedXcm) -> DispatchResult {
		let xcm_origin = T::AccountIdConverter::try_into_location(who.clone())
			.map_err(|_| Error::<T>::BadLocation)?;

		let xcm: Xcm = xcm.try_into()
			.map_err(|_| Error::<T>::BadVersion)?;

		T::XcmExecutor::execute_xcm(xcm_origin, xcm).map_err(|_| Error::<T>::ExecutionFailed.into())
	}

	fn make_xcm_upward_transfer(dest: &T::AccountId, amount: T::Balance) -> Xcm {
		Xcm::WithdrawAsset {
			assets: vec![MultiAsset::ConcreteFungible {
				id: MultiLocation::X1(Junction::Parent),
				amount: T::ToRelayChainBalance::convert(amount).into(),
			}],
			effects: vec![Order::InitiateReserveWithdraw {
				assets: vec![MultiAsset::All],
				reserve: MultiLocation::X1(Junction::Parent),
				effects: vec![Order::DepositAsset {
					assets: vec![MultiAsset::All],
					dest: MultiLocation::X1(Junction::AccountId32 {
						network: T::RelayChainNetworkId::get(),
						id: T::AccountId32Converter::convert(dest.clone()),
					}),
				}],
			}],
		}
	}

	fn make_xcm_lateral_transfer_native(
		location: MultiLocation,
		para_id: ParaId,
		network: &NetworkId,
		account: &T::AccountId,
		amount: T::Balance,
	) -> Xcm {
		Xcm::WithdrawAsset {
			assets: vec![MultiAsset::ConcreteFungible {
				id: location,
				amount: amount.into(),
			}],
			effects: vec![Order::DepositReserveAsset {
				assets: vec![MultiAsset::All],
				dest: MultiLocation::X2(Junction::Parent, Junction::Parachain { id: para_id.into() }),
				effects: vec![Order::DepositAsset {
					assets: vec![MultiAsset::All],
					dest: MultiLocation::X1(Junction::AccountId32 {
						network: network.clone(),
						id: T::AccountId32Converter::convert(account.clone()),
					}),
				}],
			}],
		}
	}

	fn make_xcm_lateral_transfer_foreign(
		reserve_chain: ParaId,
		location: MultiLocation,
		para_id: ParaId,
		network: &NetworkId,
		account: &T::AccountId,
		amount: T::Balance,
	) -> Xcm {
		let deposit_to_dest = Order::DepositAsset {
			assets: vec![MultiAsset::All],
			dest: MultiLocation::X1(Junction::AccountId32 {
				network: network.clone(),
				id: T::AccountId32Converter::convert(account.clone()),
			}),
		};

		let reserve_chain_order = if para_id == reserve_chain {
			deposit_to_dest
		} else {
			Order::DepositReserveAsset {
				assets: vec![MultiAsset::All],
				dest: MultiLocation::X2(Junction::Parent, Junction::Parachain { id: para_id.into() }),
				effects: vec![deposit_to_dest],
			}
		};

		Xcm::WithdrawAsset {
			assets: vec![MultiAsset::ConcreteFungible {
				id: location,
				amount: amount.into(),
			}],
			effects: vec![Order::InitiateReserveWithdraw {
				assets: vec![MultiAsset::All],
				reserve: MultiLocation::X2(
					Junction::Parent,
					Junction::Parachain {
						id: reserve_chain.into(),
					},
				),
				effects: vec![reserve_chain_order],
			}],
		}
	}
}
