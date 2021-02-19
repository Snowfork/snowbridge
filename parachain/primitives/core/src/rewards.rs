use frame_support::traits::{Currency, ExistenceRequirement::KeepAlive};
use frame_system::Config;
use sp_runtime::traits::Zero;
use sp_std::marker::PhantomData;


pub trait RewardRelayer<AccountId, Balance> {
	fn pay_relayer(source: &AccountId, relayer: &AccountId, reward: Balance);
}

pub struct InstantRewards<T, C>(PhantomData<(T, C)>);

impl<T, C> RewardRelayer<T::AccountId, C::Balance> for InstantRewards<T, C>
where
	T: Config,
	C: Currency<T::AccountId>
{
	fn pay_relayer(source: &T::AccountId, relayer: &T::AccountId, reward: C::Balance) {
		if reward.is_zero() {
			return;
		}

		let _ = C::transfer(
			source,
			relayer,
			reward,
			// the relayer fund account must stay above ED (needs to be pre-funded)
			KeepAlive,
		);
	}
}
