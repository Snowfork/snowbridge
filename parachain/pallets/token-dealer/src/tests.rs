use crate::mock::{new_tester, AccountId, Origin, Assets, TokenDealer, LocationConverter};
use frame_support::{assert_ok};
use sp_core::H160;

use sp_keyring::AccountKeyring as Keyring;
pub use artemis_core::assets::MultiAsset;
use xcm_executor::traits::LocationConversion;

use crate::{XAssetId, AssetId};

use xcm::v0::{MultiLocation, Junction, NetworkId};

const SIBLING_LOCATION: MultiLocation = MultiLocation::X2(Junction::Parent, Junction::Parachain { id: 200 });

// Transferring native ETH should cause the following effects locally:
// * withdrawal from local account
// * deposit into sovereign account for receipient parachain
#[test]
fn transfer_native_eth() {
	new_tester().execute_with(|| {

		let x_asset_id = XAssetId {
			reserve_chain: 100.into(),
			asset: AssetId::ETH
		};

		let network_id = NetworkId::Any;
		let sovereign_account = LocationConverter::from_location(&SIBLING_LOCATION).unwrap();

		let alice: AccountId = Keyring::Alice.into();
		let bob: AccountId = Keyring::Bob.into();

		Assets::deposit(H160::zero(), &alice, 500.into()).unwrap();

		assert_ok!(
			TokenDealer::transfer_to_parachain(
				Origin::signed(alice.clone()),
				x_asset_id,
				200.into(),
				network_id,
				bob.clone(),
				25
			)
		);

		assert_eq!(Assets::balances(H160::zero(), &alice), 475.into());
		assert_eq!(Assets::balances(H160::zero(), &sovereign_account), 25.into());
	})
}
