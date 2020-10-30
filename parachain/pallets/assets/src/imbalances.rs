// wrapping these imbalances in a private module is necessary to ensure absolute
// privacy of the inner member.
use crate::{TotalIssuance, Trait};
use frame_support::storage::StorageMap;
use frame_support::traits::{Get, Imbalance, TryDrop};
use sp_std::{marker, mem, result};
use sp_core::U256;

/// Opaque, move-only struct with private fields that serves as a token
/// denoting that funds have been created without any equal and opposite
/// accounting.
#[must_use]
pub struct PositiveImbalance<T: Trait, GetAssetId: Get<T::AssetId>>(
	U256,
	marker::PhantomData<(T, GetAssetId)>,
);

impl<T: Trait, GetAssetId: Get<T::AssetId>> PositiveImbalance<T, GetAssetId> {
	/// Create a new positive imbalance from a balance.
	pub fn new(amount: U256) -> Self {
		PositiveImbalance(amount, marker::PhantomData::<(T, GetAssetId)>)
	}
}

/// Opaque, move-only struct with private fields that serves as a token
/// denoting that funds have been destroyed without any equal and opposite
/// accounting.
#[must_use]
pub struct NegativeImbalance<T: Trait, GetAssetId: Get<T::AssetId>>(
	U256,
	marker::PhantomData<(T, GetAssetId)>,
);

impl<T: Trait, GetAssetId: Get<T::AssetId>> NegativeImbalance<T, GetAssetId> {
	/// Create a new negative imbalance from a balance.
	pub fn new(amount: U256) -> Self {
		NegativeImbalance(amount, marker::PhantomData::<(T, GetAssetId)>)
	}
}

impl<T: Trait, GetAssetId: Get<T::AssetId>> TryDrop for PositiveImbalance<T, GetAssetId> {
	fn try_drop(self) -> result::Result<(), Self> {
		self.drop_zero()
	}
}

impl<T: Trait, GetAssetId: Get<T::AssetId>> Imbalance<U256> for PositiveImbalance<T, GetAssetId> {
	type Opposite = NegativeImbalance<T, GetAssetId>;

	fn zero() -> Self {
		Self::new(U256::zero())
	}
	fn drop_zero(self) -> result::Result<(), Self> {
		if self.0.is_zero() {
			Ok(())
		} else {
			Err(self)
		}
	}
	fn split(self, amount: U256) -> (Self, Self) {
		let first = self.0.min(amount);
		let second = self.0 - first;

		mem::forget(self);
		(Self::new(first), Self::new(second))
	}
	fn merge(mut self, other: Self) -> Self {
		self.0 = self.0.saturating_add(other.0);
		mem::forget(other);

		self
	}
	fn subsume(&mut self, other: Self) {
		self.0 = self.0.saturating_add(other.0);
		mem::forget(other);
	}
	fn offset(self, other: Self::Opposite) -> result::Result<Self, Self::Opposite> {
		let (a, b) = (self.0, other.0);
		mem::forget((self, other));

		if a >= b {
			Ok(Self::new(a - b))
		} else {
			Err(NegativeImbalance::new(b - a))
		}
	}
	fn peek(&self) -> U256 {
		self.0
	}
}

impl<T: Trait, GetAssetId: Get<T::AssetId>> TryDrop for NegativeImbalance<T, GetAssetId> {
	fn try_drop(self) -> result::Result<(), Self> {
		self.drop_zero()
	}
}

impl<T: Trait, GetAssetId: Get<T::AssetId>> Imbalance<U256> for NegativeImbalance<T, GetAssetId> {
	type Opposite = PositiveImbalance<T, GetAssetId>;

	fn zero() -> Self {
		Self::new(U256::zero())
	}
	fn drop_zero(self) -> result::Result<(), Self> {
		if self.0.is_zero() {
			Ok(())
		} else {
			Err(self)
		}
	}
	fn split(self, amount: U256) -> (Self, Self) {
		let first = self.0.min(amount);
		let second = self.0 - first;

		mem::forget(self);
		(Self::new(first), Self::new(second))
	}
	fn merge(mut self, other: Self) -> Self {
		self.0 = self.0.saturating_add(other.0);
		mem::forget(other);

		self
	}
	fn subsume(&mut self, other: Self) {
		self.0 = self.0.saturating_add(other.0);
		mem::forget(other);
	}
	fn offset(self, other: Self::Opposite) -> result::Result<Self, Self::Opposite> {
		let (a, b) = (self.0, other.0);
		mem::forget((self, other));

		if a >= b {
			Ok(Self::new(a - b))
		} else {
			Err(PositiveImbalance::new(b - a))
		}
	}
	fn peek(&self) -> U256 {
		self.0
	}
}

impl<T: Trait, GetAssetId: Get<T::AssetId>> Drop for PositiveImbalance<T, GetAssetId> {
	/// Basic drop handler will just square up the total issuance.
	fn drop(&mut self) {
		<TotalIssuance<T>>::mutate(GetAssetId::get(), |v| *v = v.saturating_add(self.0));
	}
}

impl<T: Trait, GetAssetId: Get<T::AssetId>> Drop for NegativeImbalance<T, GetAssetId> {
	/// Basic drop handler will just square up the total issuance.
	fn drop(&mut self) {
		<TotalIssuance<T>>::mutate(GetAssetId::get(), |v| *v = v.saturating_sub(self.0));
	}
}
