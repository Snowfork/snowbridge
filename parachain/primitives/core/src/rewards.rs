use frame_support::debug::native;
use frame_support::traits::{Currency, ExistenceRequirement::KeepAlive, WithdrawReasons};
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
	C: Currency<T::AccountId>,
{
	fn pay_relayer(source: &T::AccountId, relayer: &T::AccountId, reward: C::Balance) {
		if reward.is_zero() {
			return;
		}

		// Using withdraw() & deposit() rather than transfer() to prevent a Transferred log from being emitted.
		// The rewards fund account must stay above ED (needs to be pre-funded)
		match C::withdraw(source, reward, WithdrawReasons::FEE, KeepAlive) {
			Ok(imbalance) => {
				C::resolve_creating(relayer, imbalance);
			}
			Err(err) => {
				native::error!("Unable to withdraw from rewards account: {:?}", err);
			}
		}
	}
}
