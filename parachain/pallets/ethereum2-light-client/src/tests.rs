use crate::mock::{new_tester};

use frame_support::assert_ok;
use sp_runtime::DispatchError;
use crate::mock::mock_verifier::{Origin, Test, Verifier};

#[test]
fn it_checks_ok() {
	new_tester::<Test>().execute_with(|| {
		assert_ok!(Ok);
	});
}
