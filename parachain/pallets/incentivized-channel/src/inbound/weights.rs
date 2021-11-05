use frame_support::weights::Weight;

pub trait WeightInfo {
	fn submit_for_local_dispatch() -> Weight;
	fn set_reward_fraction() -> Weight;
}

impl WeightInfo for () {
	fn submit_for_local_dispatch() -> Weight {
		0
	}

	fn set_reward_fraction() -> Weight {
		0
	}
}
