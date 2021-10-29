use frame_support::weights::Weight;

pub trait WeightInfo {
	fn submit() -> Weight;
}

impl WeightInfo for () {
	fn submit() -> Weight {
		0
	}
}
