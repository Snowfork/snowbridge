use frame_support::weights::Weight;

pub trait WeightInfo {
	fn import_header() -> Weight;
	fn import_header_not_new_finalized_with_max_prune() -> Weight;
	fn import_header_new_finalized_with_single_prune() -> Weight;
	fn import_header_not_new_finalized_with_single_prune() -> Weight;
}

impl WeightInfo for () {
	fn import_header() -> Weight {
		0
	}
	fn import_header_not_new_finalized_with_max_prune() -> Weight {
		0
	}
	fn import_header_new_finalized_with_single_prune() -> Weight {
		0
	}
	fn import_header_not_new_finalized_with_single_prune() -> Weight {
		0
	}
}
