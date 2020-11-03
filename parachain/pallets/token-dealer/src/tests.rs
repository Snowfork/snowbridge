use crate::mock::{new_tester, AccountId, Origin, TokenDealer};
use frame_support::{assert_ok};
use sp_keyring::AccountKeyring as Keyring;

use crate::{XAssetId, AssetId, ChainId};

use xcm::v0::NetworkId;

#[test]
fn it_executes_message() {

	let x_asset_id = XAssetId {
		reserve_chain_id: ChainId::ParaChain(100.into()),
		asset_id: CurrencyId::ETH
	};

	let network_id = NetworkId::Any;

	new_tester().execute_with(|| {
		let alice: AccountId = Keyring::Bob.into();
		let bob: AccountId = Keyring::Bob.into();

		assert_ok!(
			TokenDealer::transfer_to_parachain(
				Origin::signed(alice.clone()),
				x_asset_id,
				200.into(),
				bob,
				network_id,
				100
			)
		);
	})
}
