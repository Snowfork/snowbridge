use crate::mock::{new_tester, AccountId, Origin, DOT, Assets, TokenDealer, LocationConverter};
use frame_support::{assert_ok, traits::Currency};
use sp_core::H160;

use sp_keyring::AccountKeyring as Keyring;
pub use artemis_core::assets::MultiAsset;
use xcm_executor::traits::LocationConversion;

use crate::{XAssetId, AssetId};

use xcm::v0::{MultiLocation, Junction, NetworkId};

const SIBLING_LOCATION: MultiLocation = MultiLocation::X2(Junction::Parent, Junction::Parachain { id: 200 });

#[test]
fn transfer_dot_to_relay_chain() {
	new_tester().execute_with(|| {
		// local account on self parachain
		let alice: AccountId = Keyring::Alice.into();
		let _ = DOT::deposit_creating(&alice, (1000 as u128).into());

		// destination account on other parachain
		let bob: AccountId = Keyring::Bob.into();

		assert_ok!(
			TokenDealer::transfer_dot_to_relaychain(
				Origin::signed(alice.clone()),
				bob,
				(25 as u128).into()
			)
		);

		assert_eq!(
			DOT::total_balance(&alice),
			(975 as u128).into()
		);

	})
}

// Transferring native ETH should cause the following effects locally:
// * withdrawal from local account
// * deposit into local sovereign account for recipient parachain
#[test]
fn transfer_native_eth_to_parachain() {
	new_tester().execute_with(|| {

		// Asset identifier (ETH native to parachain registered with id 100)
		let x_asset_id = XAssetId {
			reserve_chain: 100.into(),
			asset: AssetId::ETH
		};

		let network_id = NetworkId::Any;
		let sovereign_account = LocationConverter::from_location(&SIBLING_LOCATION).unwrap();

		// local account on self parachain
		let alice: AccountId = Keyring::Alice.into();
		Assets::deposit(H160::zero(), &alice, 500.into()).unwrap();

		// destination account on other parachain
		let bob: AccountId = Keyring::Bob.into();

		// Initiate transfer of 25 ETH to bob on parachain 200
		assert_ok!(
			TokenDealer::transfer_bridged_asset_to_parachain(
				Origin::signed(alice.clone()),
				x_asset_id,
				200.into(),
				network_id,
				bob.clone(),
				25
			)
		);

		// ETH should be withdrawn from alice's account and deposited into parachain 200's sovereign account
		assert_eq!(Assets::balances(H160::zero(), &alice), 475.into());
		assert_eq!(Assets::balances(H160::zero(), &sovereign_account), 25.into());
	})
}

// Transferring foreign ETH should cause the following effects locally:
// * withdrawal from local account
#[test]
fn transfer_foreign_eth_to_parachain() {
	new_tester().execute_with(|| {

		// Asset identifier (ETH native to parachain registered with id 200)
		let x_asset_id = XAssetId {
			reserve_chain: 200.into(),
			asset: AssetId::ETH
		};

		let network_id = NetworkId::Any;

		// local account on self parachain
		let alice: AccountId = Keyring::Alice.into();
		Assets::deposit(H160::zero(), &alice, 500.into()).unwrap();


		// destination account on other parachain
		let bob: AccountId = Keyring::Bob.into();

		// Initiate transfer of 25 ETH to bob on parachain 200
		assert_ok!(
			TokenDealer::transfer_bridged_asset_to_parachain(
				Origin::signed(alice.clone()),
				x_asset_id,
				200.into(),
				network_id,
				bob.clone(),
				25
			)
		);

		// ETH should be withdrawn from alice's account
		assert_eq!(Assets::balances(H160::zero(), &alice), 475.into());
	})
}
