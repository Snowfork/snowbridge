// Copyright 2020 Parity Technologies (UK) Ltd.
use crate::{Error, mock::*};

use sp_core::H160;

use frame_support::{
	assert_ok, assert_noop,
	traits::{Contains, OnInitialize}
};

use artemis_core::Commitments;

#[test]
fn foo() {
	new_test_ext().execute_with(|| {
		CommitmentsModule::add(H160::zero(), Vec::from([0u8; 80]));

		<CommitmentsModule as OnInitialize<u64>>::on_initialize(20);

//		assert_eq!(CommitmentsModule::something(), Some(42));
	});
}
