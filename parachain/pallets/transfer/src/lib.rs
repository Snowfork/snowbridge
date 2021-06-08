#![cfg_attr(not(feature = "std"), no_std)]

use codec::Encode;
use frame_support::{
	decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult, ensure,
	traits::Get, Parameter,
};
use frame_system::ensure_signed;

use sp_runtime::traits::{
	AtLeast32BitUnsigned, Convert, MaybeSerializeDeserialize, Member, StaticLookup,
};
use sp_std::prelude::*;
use sp_std::vec;

use cumulus_primitives_core::{relay_chain::Balance as RelayChainBalance, ParaId};
use xcm::v0::{ExecuteXcm, Junction, MultiAsset, MultiLocation, NetworkId, Order, Xcm};
use xcm_executor::traits::Convert as XcmConvert;

use artemis_core::AssetId;

pub trait Config: frame_system::Config {
	type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
	type Balance: Parameter
		+ Member
		+ AtLeast32BitUnsigned
		+ Default
		+ Copy
		+ MaybeSerializeDeserialize
		+ Into<u128>;
	type ToRelayChainBalance: Convert<Self::Balance, RelayChainBalance>;
	type AccountIdConverter: XcmConvert<Self::Origin, MultiLocation>;
	type AccountId32Converter: Convert<Self::AccountId, [u8; 32]>;
	type RelayChainNetworkId: Get<NetworkId>;
	type ParaId: Get<ParaId>;
	type XcmExecutor: ExecuteXcm<Self::Call>;
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
		TransferredUpwards(AccountId, AccountId, Balance),

		/// Transferred to parachain. [asset_id, src, para_id, dest, dest_network, amount]
		Transferred(AssetId, AccountId, ParaId, AccountId, NetworkId, Balance),
	}
}

decl_error! {
	pub enum Error for Module<T: Config> {
		/// Location given was invalid or unsupported.
		BadLocation,
		/// The XCM message version is not supported.
		BadVersion,
		/// The XCM message was not executed locally
		ExecutionFailed
	}
}

decl_module! {
	pub struct Module<T: Config> for enum Call where origin: T::Origin {
		fn deposit_event() = default;

		/// Transfer DOT upwards to relay chain.
		#[weight = 10]
		pub fn transfer_upwards(origin, recipient: <T::Lookup as StaticLookup>::Source, amount: T::Balance) -> DispatchResult {
			let who = ensure_signed(origin.clone())?;

			let recipient = T::Lookup::lookup(recipient)?;

			let xcm = Self::make_xcm_upward_transfer(&recipient, amount);
			let xcm_origin = T::AccountIdConverter::convert_ref(origin.clone())
				.map_err(|_| Error::<T>::BadLocation)?;

			let outcome = T::XcmExecutor::execute_xcm(xcm_origin, xcm, 10);

			ensure!(!matches!(outcome, xcm::v0::Outcome::Error(_)), Error::<T>::ExecutionFailed);

			Self::deposit_event(Event::<T>::TransferredUpwards(who, recipient, amount));

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
		pub fn transfer(
			origin,
			asset: AssetId,
			para_id: ParaId,
			network: NetworkId,
			recipient: <T::Lookup as StaticLookup>::Source,
			amount: T::Balance
		) -> DispatchResult {
			let who = ensure_signed(origin.clone())?;

			if para_id == T::ParaId::get() {
				return Ok(());
			}

			let location = MultiLocation::X1(Junction::GeneralKey(asset.encode()));
			let recipient = T::Lookup::lookup(recipient)?;

			let xcm = Self::make_xcm_lateral_transfer(
						location,
						para_id,
						&network,
						&recipient,
						amount);

			let xcm_origin = T::AccountIdConverter::convert_ref(origin.clone())
				.map_err(|_| Error::<T>::BadLocation)?;

			let outcome = T::XcmExecutor::execute_xcm(xcm_origin, xcm, 10);

			ensure!(!matches!(outcome, xcm::v0::Outcome::Error(_)), Error::<T>::ExecutionFailed);

			Self::deposit_event(
				Event::<T>::Transferred(asset, who, para_id, recipient, network, amount)
			);

			Ok(())
		}
	}
}

impl<T: Config> Module<T> {
	// Transfer DOT upwards to relay chain
	fn make_xcm_upward_transfer(recipient: &T::AccountId, amount: T::Balance) -> Xcm<T::Call> {
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
						id: T::AccountId32Converter::convert(recipient.clone()),
					}),
				}],
			}],
		}
	}

	// Transfer bridged assets laterally to another parachain
	fn make_xcm_lateral_transfer(
		location: MultiLocation,
		para_id: ParaId,
		network: &NetworkId,
		recipient: &T::AccountId,
		amount: T::Balance,
	) -> Xcm<T::Call> {
		Xcm::WithdrawAsset {
			assets: vec![MultiAsset::ConcreteFungible {
				id: location,
				amount: amount.into(),
			}],
			effects: vec![Order::DepositReserveAsset {
				assets: vec![MultiAsset::All],
				dest: MultiLocation::X2(
					Junction::Parent,
					Junction::Parachain(para_id.into()),
				),
				effects: vec![Order::DepositAsset {
					assets: vec![MultiAsset::All],
					dest: MultiLocation::X1(Junction::AccountId32 {
						network: network.clone(),
						id: T::AccountId32Converter::convert(recipient.clone()),
					}),
				}],
			}],
		}
	}
}
