#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{decl_event, decl_error, decl_module, decl_storage,
	dispatch::DispatchResult,
	traits::Get, Parameter,
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

use artemis_core::AssetId;

/// Global identifier for a bridged ethereum asset (Within a polkadot consensus system)
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
					Junction::PalletInstance { id: 0 }, // fungible assets pallet
					Junction::GeneralIndex { id: 0 }, // ETH
				),
			AssetId::Token(key) =>
				MultiLocation::X3(
					Junction::PalletInstance { id: 0 }, // fungible assets pallet
					Junction::GeneralIndex { id: 1 }, // ERC20 token
					Junction::GeneralKey(key.to_fixed_bytes().into()) // address
				)
			}
	}
}

pub trait Config: frame_system::Config {
	type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
	type Balance: Parameter + Member + AtLeast32BitUnsigned + Default + Copy + MaybeSerializeDeserialize + Into<u128>;
	type ToRelayChainBalance: Convert<Self::Balance, RelayChainBalance>;
	type AccountIdConverter: LocationConversion<Self::AccountId>;
	type AccountId32Converter: Convert<Self::AccountId, [u8; 32]>;
	type RelayChainNetworkId: Get<NetworkId>;
	type ParaId: Get<ParaId>;
	type XcmExecutor: ExecuteXcm;
}

decl_storage! {
	trait Store for Module<T: Config> as TokenDealer {}
}

decl_event! {
	pub enum Event<T> where
		<T as frame_system::Config>::AccountId,
		<T as Config>::Balance,
	{
		/// Transferred DOT to relay chain. [src, dest, amount]
		TransferredToRelayChain(AccountId, AccountId, Balance),

		/// Transferred to parachain. [x_asset_id, src, para_id, dest, dest_network, amount]
		TransferredToParachain(XAssetId, AccountId, ParaId, AccountId, NetworkId, Balance),
	}
}

decl_error! {
	pub enum Error for Module<T: Config> {
		/// Location given was invalid or unsupported.
		BadLocation,
		/// The XCM message version is not supported.
		BadVersion,
		/// XCM execution failed
		ExecutionFailed,
	}
}

decl_module! {
	pub struct Module<T: Config> for enum Call where origin: T::Origin {
		fn deposit_event() = default;

		/// Transfer DOT upwards to relay chain.
		#[weight = 10]
		pub fn transfer_dot_to_relaychain(origin, dest: T::AccountId, amount: T::Balance) -> DispatchResult {
			let who = ensure_signed(origin.clone())?;
			let xcm = Self::make_xcm_upward_transfer(&dest, amount);

			Self::execute(&who, xcm.into())?;

			Self::deposit_event(Event::<T>::TransferredToRelayChain(who, dest, amount));

			Ok(())
		}

		/// Transfer bridged ethereum assets to a sibling parachain.
		///
		/// Bridged assets can be either native or foreign to the sending parachain.
		///
		/// # Arguments
		///
		/// * `asset`: Global identifier for a bridged asset
		/// * `para_id`: Destination parachain
		/// * `network`: Network for destination account
		/// * `account`: Destination account
		/// * `amount`: Amount to transfer
		#[weight = 10]
		pub fn transfer_bridged_asset_to_parachain(
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

impl<T: Config> Module<T> {

	// Execute the XCM message
	fn execute(who: &T::AccountId, xcm: VersionedXcm) -> DispatchResult {
		let xcm_origin = T::AccountIdConverter::try_into_location(who.clone())
			.map_err(|_| Error::<T>::BadLocation)?;

		let xcm: Xcm = xcm.try_into()
			.map_err(|_| Error::<T>::BadVersion)?;

		T::XcmExecutor::execute_xcm(xcm_origin, xcm)
			.map_err(|_| Error::<T>::ExecutionFailed.into())
	}

	// Transfer DOT upwards to relay chain
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

	// Transfer bridged assets which are native to this parachain
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

	// Transfer bridged assets which are foreign to this parachain
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
