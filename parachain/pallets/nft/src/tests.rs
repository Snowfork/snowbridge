//! Unit tests for the non-fungible-token module.

#![cfg(test)]

use super::*;
use frame_support::{assert_noop, assert_ok};
use mock::*;

#[test]
fn mint_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(NonFungibleTokenModule::next_token_id(), 0);
		let token_id = NonFungibleTokenModule::mint(&BOB, vec![1], ()).unwrap();
		assert_eq!(token_id, 0);
		assert_eq!(NonFungibleTokenModule::next_token_id(), 1);
		assert_ok!(NonFungibleTokenModule::mint(&BOB, vec![1], ()));
		assert_eq!(NonFungibleTokenModule::next_token_id(), 2);
		assert_ok!(NonFungibleTokenModule::mint(&ALICE, vec![1], ()));
		assert_eq!(NonFungibleTokenModule::next_token_id(), 3);
	});
}

#[test]
fn mint_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		NextTokenId::<Test>::mutate(|id| {
			*id = <Test as Config>::TokenId::max_value();
		});
		assert_noop!(
			NonFungibleTokenModule::mint(&BOB, vec![1], ()),
			Error::<Test>::NumOverflow
		);
	});
}

#[test]
fn burn_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(NonFungibleTokenModule::mint(&BOB, vec![1], ()));
		assert_ok!(NonFungibleTokenModule::burn(&BOB, TOKEN_ID));
	});
}

#[test]
fn burn_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		let token_id = NonFungibleTokenModule::mint(&BOB, vec![1], ()).unwrap();
		assert_eq!(token_id, 0);
		assert_noop!(
			NonFungibleTokenModule::burn(&BOB, TOKEN_ID_NOT_EXIST),
			Error::<Test>::TokenNotFound
		);

		assert_noop!(
			NonFungibleTokenModule::burn(&ALICE, TOKEN_ID),
			Error::<Test>::NoPermission
		);
	});
}

#[test]
fn transfer_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(NonFungibleTokenModule::mint(&BOB, vec![1], ()));
		assert_ok!(NonFungibleTokenModule::transfer(&BOB, &BOB, TOKEN_ID));
		assert_ok!(NonFungibleTokenModule::transfer(&BOB, &ALICE, TOKEN_ID));
		assert_ok!(NonFungibleTokenModule::transfer(&ALICE, &BOB, TOKEN_ID));
		assert!(NonFungibleTokenModule::is_owner(&BOB, TOKEN_ID));
	});
}

#[test]
fn transfer_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(NonFungibleTokenModule::mint(&BOB, vec![1], ()));
		assert_noop!(
			NonFungibleTokenModule::transfer(&BOB, &ALICE, TOKEN_ID_NOT_EXIST),
			Error::<Test>::TokenNotFound
		);
		assert_noop!(
			NonFungibleTokenModule::transfer(&ALICE, &BOB, TOKEN_ID),
			Error::<Test>::NoPermission
		);
		assert_noop!(
			NonFungibleTokenModule::transfer(&ALICE, &ALICE, TOKEN_ID),
			Error::<Test>::NoPermission
		);
	});
}
