use frame_support::{
	parameter_types,
	traits::{Everything, GenesisBuild},
};
use frame_system as system;

use crate as verifier;

pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

pub mod mock_verifier {

	use super::*;

	type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
	type Block = frame_system::mocking::MockBlock<Test>;

	frame_support::construct_runtime!(
		pub enum Test where
			Block = Block,
			NodeBlock = Block,
			UncheckedExtrinsic = UncheckedExtrinsic,
		{
			System: frame_system::{Pallet, Call, Storage, Event<T>},
			Verifier: verifier::{Pallet, Call, Config, Storage, Event<T>},
		}
	);

	impl frame_system::Config for Test {
		type Origin = Origin;
		type AccountId = AccountId;
	}

	impl verifier::Config for Test {}
}

pub fn new_tester<T: crate::Config>() -> sp_io::TestExternalities {
	new_tester_with_config::<T>(crate::GenesisConfig {})
}

pub fn new_tester_with_config<T: crate::Config>(
	config: crate::GenesisConfig,
) -> sp_io::TestExternalities {
	let mut storage = system::GenesisConfig::default().build_storage::<T>().unwrap();

	GenesisBuild::<T>::assimilate_storage(&config, &mut storage).unwrap();

	let ext: sp_io::TestExternalities = storage.into();
	ext
}
