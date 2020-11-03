#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{decl_event, decl_error, decl_module, decl_storage,
	dispatch::{DispatchResult, DispatchError},
	traits::Get, Parameter
};
use frame_system::ensure_signed;
use sp_runtime::{
	traits::{AtLeast32BitUnsigned, Convert, MaybeSerializeDeserialize, Member},
	RuntimeDebug,
};
use sp_std::convert::TryInto;
use sp_core::H160;
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

#[derive(Encode, Decode, Eq, PartialEq, Clone, Copy, RuntimeDebug)]
pub struct XDestination<AccountId> {
	para_id: ParaId,
	network: NetworkId,
	account: AccountId,
}

#[derive(Encode, Decode, Eq, PartialEq, Clone, Copy, RuntimeDebug)]
pub enum AssetId {
	ETH,
	ERC20(H160)
}

/// Identity of cross chain currency.
#[derive(Encode, Decode, Eq, PartialEq, Clone, RuntimeDebug)]
pub struct XAssetId {
	/// The reserve chain of the asset.
	pub reserve_chain: ParaId,
	/// The identity of the asset.
	pub asset: AssetId
}

impl Into<MultiLocation> for XAssetId {
	fn into(self) -> MultiLocation {
		match self.asset_id {
			AssetId::DOT => MultiLocation::X1(Junction::Parent),
			AssetId::ETH => MultiLocation::X1(Junction::Parent),
			AssetId::ERC20(addr) => {
				MultiLocation::X1(Junction::AccountKey20(addr.into()))
			}
		}
	}
}

pub trait Trait: frame_system::Trait {
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;

	type Balance: Parameter + Member + AtLeast32BitUnsigned + Default + Copy + MaybeSerializeDeserialize + Into<u128>;

	type ToRelayChainBalance: Convert<Self::Balance, RelayChainBalance>;

	/// Utility for converting from the signed origin (of type `Self::AccountId`) into a sensible
	/// `MultiLocation` ready for passing to the XCM interpreter.
	type AccountIdConverter: LocationConversion<Self::AccountId>;

	type AccountId32Converter: Convert<Self::AccountId, [u8; 32]>;

	type RelayChainNetworkId: Get<NetworkId>;

	/// Parachain ID.
	type ParaId: Get<ParaId>;

	/// The interpreter.
	type XcmExecutor: ExecuteXcm;
}

decl_storage! {
	trait Store for Module<T: Trait> as XTokens {}
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

		/// Transfer relay chain tokens to relay chain.
		#[weight = 10]
		pub fn transfer_to_relaychain(origin, dest: T::AccountId, amount: T::Balance) {

			let who = ensure_signed(origin.clone())?;
			let xcm = Self::make_xcm_upward_transfer(&dest, amount);

			Self::execute(&who, xcm.into())?;

			Self::deposit_event(Event::<T>::TransferredToRelayChain(who, dest, amount));

			Ok(())

		}

		/// Transfer assets to parachain.
		#[weight = 10]
		pub fn transfer_to_parachain(origin, x_asset_id: XAssetId, dest: XDestination<T::AccountId>, amount: T::Balance) -> DispatchResult {
			let who = ensure_signed(origin.clone())?;

			if para_id == T::ParaId::get() {
				return Ok(());
			}

			let xcm = match x_asset_id.chain {
				ChainId::Sibling(para_id) if T::ParaId::get() == para_id =>
					Self::make_xcm_lateral_transfer_native(x_asset_id.into(), dest, amount),
				ChainId::Sibling(para_id) =>
					Self::make_xcm_lateral_transfer_foreign(para_id, x_asset_id.into(), dest, amount),
			}

			Self::execute(&who, xcm.into())?;

			Self::deposit_event(
				Event::<T>::TransferredToParachain(x_asset_id, who, dest.para_id, dest.account_id, dest.network, amount),
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
										id: T::AccountId32Convert::convert(dest.account_id.clone()),
								}),
						}],
				}],
		}
	}

	fn make_xcm_lateral_transfer_native(
		location: MultiLocation,
		dest: &XDestination<T::AccountId>,
		amount: T::Balance,
	) -> Xcm {
		Xcm::WithdrawAsset {
			assets: vec![MultiAsset::ConcreteFungible {
				id: location,
				amount: amount.into(),
			}],
			effects: vec![Order::DepositReserveAsset {
				assets: vec![MultiAsset::All],
				dest: MultiLocation::X2(Junction::Parent, Junction::Parachain { id: dest.para_id.into() }),
				effects: vec![Order::DepositAsset {
					assets: vec![MultiAsset::All],
					dest: MultiLocation::X1(Junction::AccountId32 {
						network: dest.network,
						id: T::AccountId32Converter::convert(dest.account_id.clone()),
					}),
				}],
			}],
		}
	}

	fn make_xcm_lateral_transfer_foreign(
		reserve_chain: ParaId,
		location: MultiLocation,
		dest: &XDestination<T::AccountId>,
		amount: T::Balance,
	) -> Xcm {
		let deposit_to_dest = Order::DepositAsset {
			assets: vec![MultiAsset::All],
			dest: MultiLocation::X1(Junction::AccountId32 {
				network: dest.network,
				id: T::AccountId32Converter::convert(dest.account_id.clone()),
			}),
		};
		// If transfer to reserve chain, deposit to `dest` on reserve chain,
		// else deposit reserve asset.
		let reserve_chain_order = if dest.para_id == reserve_chain {
			deposit_to_dest
		} else {
			Order::DepositReserveAsset {
				assets: vec![MultiAsset::All],
				dest: MultiLocation::X2(Junction::Parent, Junction::Parachain { id: dest.para_id.into() }),
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
