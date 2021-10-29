//! Unit tests for the non-fungible-token module.

#![cfg(test)]

use super::*;
use frame_support::{assert_noop, assert_ok};
use mock::*;

#[test]
fn mint_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(TokensByOwner::<Test>::contains_key(BOB, 0), false);
		assert_eq!(NonFungibleTokenModule::is_owner(&BOB, 0), false);
		assert_eq!(NonFungibleTokenModule::next_token_id(), 0);
		let token_id = NonFungibleTokenModule::mint(&BOB, vec![1], ()).unwrap();
		assert_eq!(token_id, 0);
		assert_eq!(
			NonFungibleTokenModule::tokens(0),
			Some(TokenInfo { metadata: vec![1], owner: BOB, data: () })
		);
		assert_eq!(TokensByOwner::<Test>::contains_key(BOB, 0), true);
		assert_eq!(NonFungibleTokenModule::is_owner(&BOB, 0), true);

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
		assert_eq!(NonFungibleTokenModule::next_token_id(), <Test as Config>::TokenId::max_value());
		assert_noop!(NonFungibleTokenModule::mint(&BOB, vec![1], ()), Error::<Test>::NumOverflow);
		assert_eq!(NonFungibleTokenModule::next_token_id(), <Test as Config>::TokenId::max_value());
	});
}

#[test]
fn burn_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(TokensByOwner::<Test>::contains_key(BOB, 0), false);
		assert_eq!(Tokens::<Test>::get(0), None);

		assert_ok!(NonFungibleTokenModule::mint(&BOB, vec![1], ()));

		assert_eq!(TokensByOwner::<Test>::contains_key(BOB, 0), true);
		assert_eq!(
			Tokens::<Test>::get(0),
			Some(TokenInfo { metadata: vec![1], owner: BOB, data: () })
		);

		assert_ok!(NonFungibleTokenModule::burn(&BOB, TOKEN_ID));

		assert_eq!(TokensByOwner::<Test>::contains_key(BOB, 0), false);
		assert_eq!(Tokens::<Test>::get(0), None);
	});
}

#[test]
fn burn_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		let token_id = NonFungibleTokenModule::mint(&BOB, vec![1], ()).unwrap();
		assert_eq!(token_id, 0);

		assert_eq!(TokensByOwner::<Test>::contains_key(BOB, 0), true);
		assert_eq!(
			Tokens::<Test>::get(0),
			Some(TokenInfo { metadata: vec![1], owner: BOB, data: () })
		);

		assert_noop!(
			NonFungibleTokenModule::burn(&BOB, TOKEN_ID_NOT_EXIST),
			Error::<Test>::TokenNotFound
		);

		assert_eq!(TokensByOwner::<Test>::contains_key(BOB, 0), true);
		assert_eq!(
			Tokens::<Test>::get(0),
			Some(TokenInfo { metadata: vec![1], owner: BOB, data: () })
		);

		assert_noop!(NonFungibleTokenModule::burn(&ALICE, TOKEN_ID), Error::<Test>::NoPermission);

		assert_eq!(TokensByOwner::<Test>::contains_key(BOB, 0), true);
		assert_eq!(
			Tokens::<Test>::get(0),
			Some(TokenInfo { metadata: vec![1], owner: BOB, data: () })
		);
	});
}

#[test]
fn transfer_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(TokensByOwner::<Test>::contains_key(BOB, 0), false);
		assert_eq!(Tokens::<Test>::get(0), None);

		assert_ok!(NonFungibleTokenModule::mint(&BOB, vec![1], ()));

		assert_eq!(TokensByOwner::<Test>::contains_key(BOB, 0), true);
		assert_eq!(
			Tokens::<Test>::get(0),
			Some(TokenInfo { metadata: vec![1], owner: BOB, data: () })
		);

		assert_ok!(NonFungibleTokenModule::transfer(&BOB, &BOB, TOKEN_ID));

		assert_eq!(TokensByOwner::<Test>::contains_key(BOB, 0), true);
		assert_eq!(
			Tokens::<Test>::get(0),
			Some(TokenInfo { metadata: vec![1], owner: BOB, data: () })
		);

		assert_ok!(NonFungibleTokenModule::transfer(&BOB, &ALICE, TOKEN_ID));

		assert_eq!(TokensByOwner::<Test>::contains_key(BOB, 0), false);
		assert_eq!(TokensByOwner::<Test>::contains_key(ALICE, 0), true);
		assert_eq!(
			Tokens::<Test>::get(0),
			Some(TokenInfo { metadata: vec![1], owner: ALICE, data: () })
		);

		assert_ok!(NonFungibleTokenModule::transfer(&ALICE, &BOB, TOKEN_ID));

		assert_eq!(TokensByOwner::<Test>::contains_key(ALICE, 0), false);
		assert_eq!(TokensByOwner::<Test>::contains_key(BOB, 0), true);
		assert_eq!(
			Tokens::<Test>::get(0),
			Some(TokenInfo { metadata: vec![1], owner: BOB, data: () })
		);

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
