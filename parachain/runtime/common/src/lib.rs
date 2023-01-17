#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{dispatch::DispatchResult, parameter_types, PalletId};
use sp_core::H160;
use sp_std::marker::PhantomData;

use snowbridge_core::ChannelId;
pub struct OutboundRouter<T>(PhantomData<T>);

impl<T> snowbridge_core::OutboundRouter<T::AccountId> for OutboundRouter<T>
where
	T: basic_channel::outbound::Config + incentivized_channel::outbound::Config,
	T::AccountId: AsRef<[u8]>,
{
	fn submit(
		channel_id: ChannelId,
		who: &T::AccountId,
		target: H160,
		payload: &[u8],
	) -> DispatchResult {
		match channel_id {
			ChannelId::Basic => basic_channel::outbound::Pallet::<T>::submit(who, target, payload),
			ChannelId::Incentivized =>
				incentivized_channel::outbound::Pallet::<T>::submit(who, target, payload),
		}
	}
}

parameter_types! {
	pub const MaxMessagePayloadSize: u32 = 256;
	pub const MaxMessagesPerCommit: u32 = 20;
}

parameter_types! {
	pub const TreasuryPalletId: PalletId = PalletId(*b"s/treasy");
	pub const DotPalletId: PalletId = PalletId(*b"s/dotapp");
}

/// Money matters.
pub mod currency {
	use runtime_primitives::Balance;

	pub const UNITS: Balance = 1_000_000_000_000;
	pub const CENTS: Balance = UNITS / 100;
	pub const MILLICENTS: Balance = CENTS / 1_000;
}

/// Fee-related.
pub mod fee {
	use frame_support::weights::{
		constants::ExtrinsicBaseWeight, WeightToFeeCoefficient, WeightToFeeCoefficients,
		WeightToFeePolynomial,
	};
	use runtime_primitives::Balance;
	use smallvec::smallvec;
	pub use sp_runtime::Perbill;

	/// Handles converting a weight scalar to a fee value, based on the scale and granularity of the
	/// node's balance type.
	///
	/// This should typically create a mapping between the following ranges:
	///   - [0, MAXIMUM_BLOCK_WEIGHT]
	///   - [Balance::min, Balance::max]
	///
	/// Yet, it can be used for any other sort of change to weight-fee. Some examples being:
	///   - Setting it to `0` will essentially disable the weight fee.
	///   - Setting it to `1` will cause the literal `#[weight = x]` values to be charged.
	pub struct WeightToFee;
	impl WeightToFeePolynomial for WeightToFee {
		type Balance = Balance;
		fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
			// extrinsic base weight map to 1/100 CENT
			let p = super::currency::CENTS;
			let q = 100 * Balance::from(ExtrinsicBaseWeight::get().ref_time());
			smallvec![WeightToFeeCoefficient {
				degree: 1,
				negative: false,
				coeff_frac: Perbill::from_rational(p % q, q),
				coeff_integer: p / q,
			}]
		}
	}
}

#[cfg(test)]
mod tests {
	use super::{
		currency::{CENTS, MILLICENTS},
		fee::WeightToFee,
	};
	use frame_support::weights::{constants::ExtrinsicBaseWeight, WeightToFee as WeightToFeeT};

	#[test]
	// This function tests that the fee for `ExtrinsicBaseWeight` of weight is correct
	fn extrinsic_base_fee_is_correct() {
		// `ExtrinsicBaseWeight` should cost 1/100 of a CENT
		println!("Base Weight: {}", ExtrinsicBaseWeight::get());
		let x = WeightToFee::weight_to_fee(&ExtrinsicBaseWeight::get());
		println!("Base Fee: {}", x);
		let y = CENTS / 100;
		println!("CENTS: {}", y);
		assert!(x.max(y) - x.min(y) < MILLICENTS);
	}
}
