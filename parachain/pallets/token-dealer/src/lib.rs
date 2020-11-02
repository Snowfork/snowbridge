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
pub enum CurrencyId {
	DOT,
	ETH,
	ERC20(H160)
}

#[derive(Encode, Decode, Eq, PartialEq, Clone, RuntimeDebug)]
/// Identity of chain.
pub enum ChainId {
	/// The relay chain.
	RelayChain,
	/// A parachain.
	ParaChain(ParaId),
}

#[derive(Encode, Decode, Eq, PartialEq, Clone, RuntimeDebug)]
/// Identity of cross chain currency.
pub struct XCurrencyId {
	/// The reserve chain of the currency.
	pub chain_id: ChainId,
	/// The identity of the currency.
	pub currency_id: CurrencyId
}

impl Into<MultiLocation> for XCurrencyId {
	fn into(self) -> MultiLocation {
		match self {
			XCurrencyId { currency_id: CurrencyId::DOT , .. } => {
				MultiLocation::X1(Junction::GeneralIndex { id: 2})
			},
			_ => {
				MultiLocation::X1(Junction::GeneralIndex { id: 2})
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

		/// Transferred to parachain. [x_currency_id, src, para_id, dest, dest_network, amount]
		TransferredToParachain(XCurrencyId, AccountId, ParaId, AccountId, NetworkId, Balance),
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

		/// Transfer tokens to parachain.
		#[weight = 10]
		pub fn transfer_to_parachain(
			origin,
			x_currency_id: XCurrencyId,
			para_id: ParaId,
			dest: T::AccountId,
			dest_network: NetworkId,
			amount: T::Balance) -> DispatchResult
		{
			let who = ensure_signed(origin.clone())?;

			if para_id == T::ParaId::get() {
				return Ok(());
			}

			let xcm = Self::do_transfer_to_parachain(
				x_currency_id.clone(),
				para_id,
				&dest,
				dest_network.clone(),
				amount,
			).ok_or(DispatchError::Other("Transfer type not supported"))?;

			Self::execute(&who, xcm.into())?;

			Self::deposit_event(
				Event::<T>::TransferredToParachain(x_currency_id, who, para_id, dest, dest_network, amount),
			);

			Ok(())
		}
	}
}

impl<T: Trait> Module<T> {

	fn execute(who: &T::AccountId, xcm: VersionedXcm) -> DispatchResult {
		let xcm_origin = T::AccountIdConverter::try_into_location(who.clone())
			.map_err(|_| Error::<T>::BadLocation)?;

		let xcm = xcm.try_into().map_err(|_| Error::<T>::BadVersion)?;
		T::XcmExecutor::execute_xcm(xcm_origin, xcm).map_err(|_| Error::<T>::ExecutionFailed.into())
	}

	fn do_transfer_to_parachain(
		x_currency_id: XCurrencyId,
		para_id: ParaId,
		dest: &T::AccountId,
		dest_network: NetworkId,
		amount: T::Balance,
	) -> Option<Xcm> {
		match x_currency_id.chain_id {
			ChainId::RelayChain => None,
			ChainId::ParaChain(reserve_chain) => {
				if T::ParaId::get() == reserve_chain {
					Some(Self::transfer_owned_tokens_to_parachain(x_currency_id, para_id, dest, dest_network, amount))
				} else {
					Some(Self::transfer_non_owned_tokens_to_parachain(
						reserve_chain,
						x_currency_id,
						para_id,
						dest,
						dest_network,
						amount,
					))
				}
			}
		}
	}

	/// Transfer parachain tokens "owned" by self parachain to another
	/// parachain.
	///
	/// NOTE - `para_id` must not be self parachain.
	fn transfer_owned_tokens_to_parachain(
		x_currency_id: XCurrencyId,
		para_id: ParaId,
		dest: &T::AccountId,
		dest_network: NetworkId,
		amount: T::Balance,
	) -> Xcm {
		Xcm::WithdrawAsset {
			assets: vec![MultiAsset::ConcreteFungible {
				id: x_currency_id.into(),
				amount: amount.into(),
			}],
			effects: vec![Order::DepositReserveAsset {
				assets: vec![MultiAsset::All],
				dest: MultiLocation::X2(Junction::Parent, Junction::Parachain { id: para_id.into() }),
				effects: vec![Order::DepositAsset {
					assets: vec![MultiAsset::All],
					dest: MultiLocation::X1(Junction::AccountId32 {
						network: dest_network,
						id: T::AccountId32Converter::convert(dest.clone()),
					}),
				}],
			}],
		}
	}

	/// Transfer parachain tokens not "owned" by self chain to another
	/// parachain.
	fn transfer_non_owned_tokens_to_parachain(
		reserve_chain: ParaId,
		x_currency_id: XCurrencyId,
		para_id: ParaId,
		dest: &T::AccountId,
		dest_network: NetworkId,
		amount: T::Balance,
	) -> Xcm {
		let deposit_to_dest = Order::DepositAsset {
			assets: vec![MultiAsset::All],
			dest: MultiLocation::X1(Junction::AccountId32 {
				network: dest_network,
				id: T::AccountId32Converter::convert(dest.clone()),
			}),
		};
		// If transfer to reserve chain, deposit to `dest` on reserve chain,
		// else deposit reserve asset.
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
				id: x_currency_id.into(),
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
