use frame_support::weights::Weight;

pub trait WeightInfo {
    fn submit() -> Weight;
}

// TODO: placeholder implementation, generate weights properly
impl WeightInfo for () {
    fn submit() -> Weight {
        42
    }
}
