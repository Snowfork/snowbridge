use frame_support::weights::Weight;

pub trait WeightInfo {
	fn lock() -> Weight;
	fn unlock() -> Weight;
}

impl WeightInfo for () {
	fn lock() -> Weight {
		0
	}
	fn unlock() -> Weight {
		0
	}
}
