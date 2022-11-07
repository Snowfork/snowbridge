use frame_support::weights::Weight;

pub trait WeightInfo {
    fn verify_message() -> Weight;
}

// TODO: placeholder implementation, generate weights properly
impl WeightInfo for () {
    fn verify_message() -> Weight {
        Weight::from_ref_time(42u64)
    }
}
