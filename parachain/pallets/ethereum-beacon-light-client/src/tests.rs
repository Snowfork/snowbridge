use crate::mock::*;
use crate::Error;
use frame_support::{assert_ok};

#[test]
fn it_syncs_from_an_initial_checkpoint() {
	let initial_sync = get_initial_sync();

	new_tester().execute_with(|| {
		assert_ok!(
			EthereumBeaconLightClient::initial_sync(
				Origin::signed(1),
				initial_sync,
			)
		);
	});
}

