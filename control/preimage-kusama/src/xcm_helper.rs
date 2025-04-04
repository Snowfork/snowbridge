use frame_support::parameter_types;
use xcm::latest::prelude::*;
use xcm_builder::GlobalConsensusParachainConvertsFor;
use xcm_executor::traits::ConvertLocation;

parameter_types! {
    pub LocalUniversalLocation: InteriorLocation = [
       GlobalConsensus(Kusama),
      Parachain(1000),
   ].into();
}

pub fn get_pah_owner_on_kusama() -> [u8; 32] {
    let pah_location = Location {
        parents: 2,
        interior: [GlobalConsensus(Polkadot), Parachain(1000)].into(),
    };
    let pah_sovereign =
        GlobalConsensusParachainConvertsFor::<LocalUniversalLocation, [u8; 32]>::convert_location(
            &pah_location,
        )
        .unwrap();

    pah_sovereign
}
