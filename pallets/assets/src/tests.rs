// This file is part of Substrate.

// Copyright (C) 2019-2022 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Tests for Assets pallet.

use super::*;
use super::traits::OmniverseTokenFactoryHandler;
use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_err, assert_ok, traits::Currency};
use pallet_balances::Error as BalancesError;
use sp_runtime::{traits::ConvertInto, TokenError};
use pallet_omniverse_protocol::{traits::OmniverseAccounts, OmniverseTokenProtocol, VerifyResult, VerifyError, TokenOpcode, TransferTokenOp, MintTokenOp, TRANSFER, MINT};
use sha3::{Digest, Keccak256};
use secp256k1::rand::rngs::OsRng;
use secp256k1::{Secp256k1, Message, ecdsa::RecoverableSignature, SecretKey, PublicKey};
use codec::{Encode, Decode};

#[test]
fn basic_minting_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::force_create(RuntimeOrigin::root(), 0, 1, true, 1));
		assert_ok!(Assets::mint(RuntimeOrigin::signed(1), 0, 1, 100));
		assert_eq!(Assets::balance(0, 1), 100);
		assert_ok!(Assets::mint(RuntimeOrigin::signed(1), 0, 2, 100));
		assert_eq!(Assets::balance(0, 2), 100);
	});
}

#[test]
fn minting_too_many_insufficient_assets_fails() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::force_create(RuntimeOrigin::root(), 0, 1, false, 1));
		assert_ok!(Assets::force_create(RuntimeOrigin::root(), 1, 1, false, 1));
		assert_ok!(Assets::force_create(RuntimeOrigin::root(), 2, 1, false, 1));
		Balances::make_free_balance_be(&1, 100);
		assert_ok!(Assets::mint(RuntimeOrigin::signed(1), 0, 1, 100));
		assert_ok!(Assets::mint(RuntimeOrigin::signed(1), 1, 1, 100));
		assert_noop!(Assets::mint(RuntimeOrigin::signed(1), 2, 1, 100), TokenError::CannotCreate);

		Balances::make_free_balance_be(&2, 1);
		assert_ok!(Assets::transfer(RuntimeOrigin::signed(1), 0, 2, 100));
		assert_ok!(Assets::mint(RuntimeOrigin::signed(1), 2, 1, 100));
	});
}

#[test]
fn minting_insufficient_asset_with_deposit_should_work_when_consumers_exhausted() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::force_create(RuntimeOrigin::root(), 0, 1, false, 1));
		assert_ok!(Assets::force_create(RuntimeOrigin::root(), 1, 1, false, 1));
		assert_ok!(Assets::force_create(RuntimeOrigin::root(), 2, 1, false, 1));
		Balances::make_free_balance_be(&1, 100);
		assert_ok!(Assets::mint(RuntimeOrigin::signed(1), 0, 1, 100));
		assert_ok!(Assets::mint(RuntimeOrigin::signed(1), 1, 1, 100));
		assert_noop!(Assets::mint(RuntimeOrigin::signed(1), 2, 1, 100), TokenError::CannotCreate);

		assert_ok!(Assets::touch(RuntimeOrigin::signed(1), 2));
		assert_eq!(Balances::reserved_balance(&1), 10);

		assert_ok!(Assets::mint(RuntimeOrigin::signed(1), 2, 1, 100));
	});
}

#[test]
fn minting_insufficient_assets_with_deposit_without_consumer_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::force_create(RuntimeOrigin::root(), 0, 1, false, 1));
		assert_noop!(Assets::mint(RuntimeOrigin::signed(1), 0, 1, 100), TokenError::CannotCreate);
		Balances::make_free_balance_be(&1, 100);
		assert_ok!(Assets::touch(RuntimeOrigin::signed(1), 0));
		assert_ok!(Assets::mint(RuntimeOrigin::signed(1), 0, 1, 100));
		assert_eq!(Balances::reserved_balance(&1), 10);
		assert_eq!(System::consumers(&1), 0);
	});
}

#[test]
fn refunding_asset_deposit_with_burn_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::force_create(RuntimeOrigin::root(), 0, 1, false, 1));
		Balances::make_free_balance_be(&1, 100);
		assert_ok!(Assets::touch(RuntimeOrigin::signed(1), 0));
		assert_ok!(Assets::mint(RuntimeOrigin::signed(1), 0, 1, 100));
		assert_ok!(Assets::refund(RuntimeOrigin::signed(1), 0, true));
		assert_eq!(Balances::reserved_balance(&1), 0);
		assert_eq!(Assets::balance(1, 0), 0);
	});
}

#[test]
fn refunding_asset_deposit_with_burn_disallowed_should_fail() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::force_create(RuntimeOrigin::root(), 0, 1, false, 1));
		Balances::make_free_balance_be(&1, 100);
		assert_ok!(Assets::touch(RuntimeOrigin::signed(1), 0));
		assert_ok!(Assets::mint(RuntimeOrigin::signed(1), 0, 1, 100));
		assert_noop!(Assets::refund(RuntimeOrigin::signed(1), 0, false), Error::<Test>::WouldBurn);
	});
}

#[test]
fn refunding_asset_deposit_without_burn_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::force_create(RuntimeOrigin::root(), 0, 1, false, 1));
		assert_noop!(Assets::mint(RuntimeOrigin::signed(1), 0, 1, 100), TokenError::CannotCreate);
		Balances::make_free_balance_be(&1, 100);
		assert_ok!(Assets::touch(RuntimeOrigin::signed(1), 0));
		assert_ok!(Assets::mint(RuntimeOrigin::signed(1), 0, 1, 100));
		Balances::make_free_balance_be(&2, 100);
		assert_ok!(Assets::transfer(RuntimeOrigin::signed(1), 0, 2, 100));
		assert_eq!(Assets::balance(0, 2), 100);
		assert_eq!(Assets::balance(0, 1), 0);
		assert_eq!(Balances::reserved_balance(&1), 10);
		assert_ok!(Assets::refund(RuntimeOrigin::signed(1), 0, false));
		assert_eq!(Balances::reserved_balance(&1), 0);
		assert_eq!(Assets::balance(1, 0), 0);
	});
}

/// Refunding reaps an account and calls the `FrozenBalance::died` hook.
#[test]
fn refunding_calls_died_hook() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::force_create(RuntimeOrigin::root(), 0, 1, false, 1));
		Balances::make_free_balance_be(&1, 100);
		assert_ok!(Assets::touch(RuntimeOrigin::signed(1), 0));
		assert_ok!(Assets::mint(RuntimeOrigin::signed(1), 0, 1, 100));
		assert_ok!(Assets::refund(RuntimeOrigin::signed(1), 0, true));

		assert_eq!(Asset::<Test>::get(0).unwrap().accounts, 0);
		assert_eq!(hooks(), vec![Hook::Died(0, 1)]);
	});
}

#[test]
fn approval_lifecycle_works() {
	new_test_ext().execute_with(|| {
		// can't approve non-existent token
		assert_noop!(
			Assets::approve_transfer(RuntimeOrigin::signed(1), 0, 2, 50),
			Error::<Test>::Unknown
		);
		// so we create it :)
		assert_ok!(Assets::force_create(RuntimeOrigin::root(), 0, 1, true, 1));
		assert_ok!(Assets::mint(RuntimeOrigin::signed(1), 0, 1, 100));
		Balances::make_free_balance_be(&1, 1);
		assert_ok!(Assets::approve_transfer(RuntimeOrigin::signed(1), 0, 2, 50));
		assert_eq!(Asset::<Test>::get(0).unwrap().approvals, 1);
		assert_eq!(Balances::reserved_balance(&1), 1);
		assert_ok!(Assets::transfer_approved(RuntimeOrigin::signed(2), 0, 1, 3, 40));
		assert_eq!(Asset::<Test>::get(0).unwrap().approvals, 1);
		assert_ok!(Assets::cancel_approval(RuntimeOrigin::signed(1), 0, 2));
		assert_eq!(Asset::<Test>::get(0).unwrap().approvals, 0);
		assert_eq!(Assets::balance(0, 1), 60);
		assert_eq!(Assets::balance(0, 3), 40);
		assert_eq!(Balances::reserved_balance(&1), 0);
	});
}

#[test]
fn transfer_approved_all_funds() {
	new_test_ext().execute_with(|| {
		// can't approve non-existent token
		assert_noop!(
			Assets::approve_transfer(RuntimeOrigin::signed(1), 0, 2, 50),
			Error::<Test>::Unknown
		);
		// so we create it :)
		assert_ok!(Assets::force_create(RuntimeOrigin::root(), 0, 1, true, 1));
		assert_ok!(Assets::mint(RuntimeOrigin::signed(1), 0, 1, 100));
		Balances::make_free_balance_be(&1, 1);
		assert_ok!(Assets::approve_transfer(RuntimeOrigin::signed(1), 0, 2, 50));
		assert_eq!(Asset::<Test>::get(0).unwrap().approvals, 1);
		assert_eq!(Balances::reserved_balance(&1), 1);

		// transfer the full amount, which should trigger auto-cleanup
		assert_ok!(Assets::transfer_approved(RuntimeOrigin::signed(2), 0, 1, 3, 50));
		assert_eq!(Asset::<Test>::get(0).unwrap().approvals, 0);
		assert_eq!(Assets::balance(0, 1), 50);
		assert_eq!(Assets::balance(0, 3), 50);
		assert_eq!(Balances::reserved_balance(&1), 0);
	});
}

#[test]
fn approval_deposits_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::force_create(RuntimeOrigin::root(), 0, 1, true, 1));
		assert_ok!(Assets::mint(RuntimeOrigin::signed(1), 0, 1, 100));
		let e = BalancesError::<Test>::InsufficientBalance;
		assert_noop!(Assets::approve_transfer(RuntimeOrigin::signed(1), 0, 2, 50), e);

		Balances::make_free_balance_be(&1, 1);
		assert_ok!(Assets::approve_transfer(RuntimeOrigin::signed(1), 0, 2, 50));
		assert_eq!(Balances::reserved_balance(&1), 1);

		assert_ok!(Assets::transfer_approved(RuntimeOrigin::signed(2), 0, 1, 3, 50));
		assert_eq!(Balances::reserved_balance(&1), 0);

		assert_ok!(Assets::approve_transfer(RuntimeOrigin::signed(1), 0, 2, 50));
		assert_ok!(Assets::cancel_approval(RuntimeOrigin::signed(1), 0, 2));
		assert_eq!(Balances::reserved_balance(&1), 0);
	});
}

#[test]
fn cannot_transfer_more_than_approved() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::force_create(RuntimeOrigin::root(), 0, 1, true, 1));
		assert_ok!(Assets::mint(RuntimeOrigin::signed(1), 0, 1, 100));
		Balances::make_free_balance_be(&1, 1);
		assert_ok!(Assets::approve_transfer(RuntimeOrigin::signed(1), 0, 2, 50));
		let e = Error::<Test>::Unapproved;
		assert_noop!(Assets::transfer_approved(RuntimeOrigin::signed(2), 0, 1, 3, 51), e);
	});
}

#[test]
fn cannot_transfer_more_than_exists() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::force_create(RuntimeOrigin::root(), 0, 1, true, 1));
		assert_ok!(Assets::mint(RuntimeOrigin::signed(1), 0, 1, 100));
		Balances::make_free_balance_be(&1, 1);
		assert_ok!(Assets::approve_transfer(RuntimeOrigin::signed(1), 0, 2, 101));
		let e = Error::<Test>::BalanceLow;
		assert_noop!(Assets::transfer_approved(RuntimeOrigin::signed(2), 0, 1, 3, 101), e);
	});
}

#[test]
fn cancel_approval_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::force_create(RuntimeOrigin::root(), 0, 1, true, 1));
		assert_ok!(Assets::mint(RuntimeOrigin::signed(1), 0, 1, 100));
		Balances::make_free_balance_be(&1, 1);
		assert_ok!(Assets::approve_transfer(RuntimeOrigin::signed(1), 0, 2, 50));
		assert_eq!(Asset::<Test>::get(0).unwrap().approvals, 1);
		assert_noop!(
			Assets::cancel_approval(RuntimeOrigin::signed(1), 1, 2),
			Error::<Test>::Unknown
		);
		assert_noop!(
			Assets::cancel_approval(RuntimeOrigin::signed(2), 0, 2),
			Error::<Test>::Unknown
		);
		assert_noop!(
			Assets::cancel_approval(RuntimeOrigin::signed(1), 0, 3),
			Error::<Test>::Unknown
		);
		assert_eq!(Asset::<Test>::get(0).unwrap().approvals, 1);
		assert_ok!(Assets::cancel_approval(RuntimeOrigin::signed(1), 0, 2));
		assert_eq!(Asset::<Test>::get(0).unwrap().approvals, 0);
		assert_noop!(
			Assets::cancel_approval(RuntimeOrigin::signed(1), 0, 2),
			Error::<Test>::Unknown
		);
	});
}

#[test]
fn force_cancel_approval_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::force_create(RuntimeOrigin::root(), 0, 1, true, 1));
		assert_ok!(Assets::mint(RuntimeOrigin::signed(1), 0, 1, 100));
		Balances::make_free_balance_be(&1, 1);
		assert_ok!(Assets::approve_transfer(RuntimeOrigin::signed(1), 0, 2, 50));
		assert_eq!(Asset::<Test>::get(0).unwrap().approvals, 1);
		let e = Error::<Test>::NoPermission;
		assert_noop!(Assets::force_cancel_approval(RuntimeOrigin::signed(2), 0, 1, 2), e);
		assert_noop!(
			Assets::force_cancel_approval(RuntimeOrigin::signed(1), 1, 1, 2),
			Error::<Test>::Unknown
		);
		assert_noop!(
			Assets::force_cancel_approval(RuntimeOrigin::signed(1), 0, 2, 2),
			Error::<Test>::Unknown
		);
		assert_noop!(
			Assets::force_cancel_approval(RuntimeOrigin::signed(1), 0, 1, 3),
			Error::<Test>::Unknown
		);
		assert_eq!(Asset::<Test>::get(0).unwrap().approvals, 1);
		assert_ok!(Assets::force_cancel_approval(RuntimeOrigin::signed(1), 0, 1, 2));
		assert_eq!(Asset::<Test>::get(0).unwrap().approvals, 0);
		assert_noop!(
			Assets::force_cancel_approval(RuntimeOrigin::signed(1), 0, 1, 2),
			Error::<Test>::Unknown
		);
	});
}

#[test]
fn lifecycle_should_work() {
	new_test_ext().execute_with(|| {
		Balances::make_free_balance_be(&1, 100);
		assert_ok!(Assets::create(RuntimeOrigin::signed(1), 0, 1, 1));
		assert_eq!(Balances::reserved_balance(&1), 1);
		assert!(Asset::<Test>::contains_key(0));

		assert_ok!(Assets::set_metadata(RuntimeOrigin::signed(1), 0, vec![0], vec![0], 12));
		assert_eq!(Balances::reserved_balance(&1), 4);
		assert!(Metadata::<Test>::contains_key(0));

		Balances::make_free_balance_be(&10, 100);
		assert_ok!(Assets::mint(RuntimeOrigin::signed(1), 0, 10, 100));
		Balances::make_free_balance_be(&20, 100);
		assert_ok!(Assets::mint(RuntimeOrigin::signed(1), 0, 20, 100));
		assert_eq!(Account::<Test>::iter_prefix(0).count(), 2);

		let w = Asset::<Test>::get(0).unwrap().destroy_witness();
		assert_ok!(Assets::destroy(RuntimeOrigin::signed(1), 0, w));
		assert_eq!(Balances::reserved_balance(&1), 0);

		assert!(!Asset::<Test>::contains_key(0));
		assert!(!Metadata::<Test>::contains_key(0));
		assert_eq!(Account::<Test>::iter_prefix(0).count(), 0);

		assert_ok!(Assets::create(RuntimeOrigin::signed(1), 0, 1, 1));
		assert_eq!(Balances::reserved_balance(&1), 1);
		assert!(Asset::<Test>::contains_key(0));

		assert_ok!(Assets::set_metadata(RuntimeOrigin::signed(1), 0, vec![0], vec![0], 12));
		assert_eq!(Balances::reserved_balance(&1), 4);
		assert!(Metadata::<Test>::contains_key(0));

		assert_ok!(Assets::mint(RuntimeOrigin::signed(1), 0, 10, 100));
		assert_ok!(Assets::mint(RuntimeOrigin::signed(1), 0, 20, 100));
		assert_eq!(Account::<Test>::iter_prefix(0).count(), 2);

		let w = Asset::<Test>::get(0).unwrap().destroy_witness();
		assert_ok!(Assets::destroy(RuntimeOrigin::root(), 0, w));
		assert_eq!(Balances::reserved_balance(&1), 0);

		assert!(!Asset::<Test>::contains_key(0));
		assert!(!Metadata::<Test>::contains_key(0));
		assert_eq!(Account::<Test>::iter_prefix(0).count(), 0);
	});
}

#[test]
fn destroy_with_bad_witness_should_not_work() {
	new_test_ext().execute_with(|| {
		Balances::make_free_balance_be(&1, 100);
		assert_ok!(Assets::force_create(RuntimeOrigin::root(), 0, 1, true, 1));
		let mut w = Asset::<Test>::get(0).unwrap().destroy_witness();
		assert_ok!(Assets::mint(RuntimeOrigin::signed(1), 0, 10, 100));
		// witness too low
		assert_noop!(Assets::destroy(RuntimeOrigin::signed(1), 0, w), Error::<Test>::BadWitness);
		// witness too high is okay though
		w.accounts += 2;
		w.sufficients += 2;
		assert_ok!(Assets::destroy(RuntimeOrigin::signed(1), 0, w));
	});
}

#[test]
fn destroy_should_refund_approvals() {
	new_test_ext().execute_with(|| {
		Balances::make_free_balance_be(&1, 100);
		assert_ok!(Assets::force_create(RuntimeOrigin::root(), 0, 1, true, 1));
		assert_ok!(Assets::mint(RuntimeOrigin::signed(1), 0, 10, 100));
		assert_ok!(Assets::approve_transfer(RuntimeOrigin::signed(1), 0, 2, 50));
		assert_ok!(Assets::approve_transfer(RuntimeOrigin::signed(1), 0, 3, 50));
		assert_ok!(Assets::approve_transfer(RuntimeOrigin::signed(1), 0, 4, 50));
		assert_eq!(Balances::reserved_balance(&1), 3);

		let w = Asset::<Test>::get(0).unwrap().destroy_witness();
		assert_ok!(Assets::destroy(RuntimeOrigin::signed(1), 0, w));
		assert_eq!(Balances::reserved_balance(&1), 0);

		// all approvals are removed
		assert!(Approvals::<Test>::iter().count().is_zero())
	});
}

#[test]
fn non_providing_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::force_create(RuntimeOrigin::root(), 0, 1, false, 1));

		Balances::make_free_balance_be(&0, 100);
		assert_ok!(Assets::mint(RuntimeOrigin::signed(1), 0, 0, 100));

		// Cannot mint into account 2 since it doesn't (yet) exist...
		assert_noop!(Assets::mint(RuntimeOrigin::signed(1), 0, 1, 100), TokenError::CannotCreate);
		// ...or transfer...
		assert_noop!(
			Assets::transfer(RuntimeOrigin::signed(0), 0, 1, 50),
			TokenError::CannotCreate
		);
		// ...or force-transfer
		assert_noop!(
			Assets::force_transfer(RuntimeOrigin::signed(1), 0, 0, 1, 50),
			TokenError::CannotCreate
		);

		Balances::make_free_balance_be(&1, 100);
		Balances::make_free_balance_be(&2, 100);
		assert_ok!(Assets::transfer(RuntimeOrigin::signed(0), 0, 1, 25));
		assert_ok!(Assets::force_transfer(RuntimeOrigin::signed(1), 0, 0, 2, 25));
	});
}

#[test]
fn min_balance_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::force_create(RuntimeOrigin::root(), 0, 1, true, 10));
		assert_ok!(Assets::mint(RuntimeOrigin::signed(1), 0, 1, 100));
		assert_eq!(Asset::<Test>::get(0).unwrap().accounts, 1);

		// Cannot create a new account with a balance that is below minimum...
		assert_noop!(Assets::mint(RuntimeOrigin::signed(1), 0, 2, 9), TokenError::BelowMinimum);
		assert_noop!(Assets::transfer(RuntimeOrigin::signed(1), 0, 2, 9), TokenError::BelowMinimum);
		assert_noop!(
			Assets::force_transfer(RuntimeOrigin::signed(1), 0, 1, 2, 9),
			TokenError::BelowMinimum
		);

		// When deducting from an account to below minimum, it should be reaped.
		// Death by `transfer`.
		assert_ok!(Assets::transfer(RuntimeOrigin::signed(1), 0, 2, 91));
		assert!(Assets::maybe_balance(0, 1).is_none());
		assert_eq!(Assets::balance(0, 2), 100);
		assert_eq!(Asset::<Test>::get(0).unwrap().accounts, 1);
		assert_eq!(take_hooks(), vec![Hook::Died(0, 1)]);

		// Death by `force_transfer`.
		assert_ok!(Assets::force_transfer(RuntimeOrigin::signed(1), 0, 2, 1, 91));
		assert!(Assets::maybe_balance(0, 2).is_none());
		assert_eq!(Assets::balance(0, 1), 100);
		assert_eq!(Asset::<Test>::get(0).unwrap().accounts, 1);
		assert_eq!(take_hooks(), vec![Hook::Died(0, 2)]);

		// Death by `burn`.
		assert_ok!(Assets::burn(RuntimeOrigin::signed(1), 0, 1, 91));
		assert!(Assets::maybe_balance(0, 1).is_none());
		assert_eq!(Asset::<Test>::get(0).unwrap().accounts, 0);
		assert_eq!(take_hooks(), vec![Hook::Died(0, 1)]);

		// Death by `transfer_approved`.
		assert_ok!(Assets::mint(RuntimeOrigin::signed(1), 0, 1, 100));
		Balances::make_free_balance_be(&1, 1);
		assert_ok!(Assets::approve_transfer(RuntimeOrigin::signed(1), 0, 2, 100));
		assert_ok!(Assets::transfer_approved(RuntimeOrigin::signed(2), 0, 1, 3, 91));
		assert_eq!(take_hooks(), vec![Hook::Died(0, 1)]);
	});
}

#[test]
fn querying_total_supply_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::force_create(RuntimeOrigin::root(), 0, 1, true, 1));
		assert_ok!(Assets::mint(RuntimeOrigin::signed(1), 0, 1, 100));
		assert_eq!(Assets::balance(0, 1), 100);
		assert_ok!(Assets::transfer(RuntimeOrigin::signed(1), 0, 2, 50));
		assert_eq!(Assets::balance(0, 1), 50);
		assert_eq!(Assets::balance(0, 2), 50);
		assert_ok!(Assets::transfer(RuntimeOrigin::signed(2), 0, 3, 31));
		assert_eq!(Assets::balance(0, 1), 50);
		assert_eq!(Assets::balance(0, 2), 19);
		assert_eq!(Assets::balance(0, 3), 31);
		assert_ok!(Assets::burn(RuntimeOrigin::signed(1), 0, 3, u128::MAX));
		assert_eq!(Assets::total_supply(0), 69);
	});
}

#[test]
fn transferring_amount_below_available_balance_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::force_create(RuntimeOrigin::root(), 0, 1, true, 1));
		assert_ok!(Assets::mint(RuntimeOrigin::signed(1), 0, 1, 100));
		assert_eq!(Assets::balance(0, 1), 100);
		assert_ok!(Assets::transfer(RuntimeOrigin::signed(1), 0, 2, 50));
		assert_eq!(Assets::balance(0, 1), 50);
		assert_eq!(Assets::balance(0, 2), 50);
	});
}

#[test]
fn transferring_enough_to_kill_source_when_keep_alive_should_fail() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::force_create(RuntimeOrigin::root(), 0, 1, true, 10));
		assert_ok!(Assets::mint(RuntimeOrigin::signed(1), 0, 1, 100));
		assert_eq!(Assets::balance(0, 1), 100);
		assert_noop!(
			Assets::transfer_keep_alive(RuntimeOrigin::signed(1), 0, 2, 91),
			Error::<Test>::BalanceLow
		);
		assert_ok!(Assets::transfer_keep_alive(RuntimeOrigin::signed(1), 0, 2, 90));
		assert_eq!(Assets::balance(0, 1), 10);
		assert_eq!(Assets::balance(0, 2), 90);
		assert!(hooks().is_empty());
	});
}

#[test]
fn transferring_frozen_user_should_not_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::force_create(RuntimeOrigin::root(), 0, 1, true, 1));
		assert_ok!(Assets::mint(RuntimeOrigin::signed(1), 0, 1, 100));
		assert_eq!(Assets::balance(0, 1), 100);
		assert_ok!(Assets::freeze(RuntimeOrigin::signed(1), 0, 1));
		assert_noop!(Assets::transfer(RuntimeOrigin::signed(1), 0, 2, 50), Error::<Test>::Frozen);
		assert_ok!(Assets::thaw(RuntimeOrigin::signed(1), 0, 1));
		assert_ok!(Assets::transfer(RuntimeOrigin::signed(1), 0, 2, 50));
	});
}

#[test]
fn transferring_frozen_asset_should_not_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::force_create(RuntimeOrigin::root(), 0, 1, true, 1));
		assert_ok!(Assets::mint(RuntimeOrigin::signed(1), 0, 1, 100));
		assert_eq!(Assets::balance(0, 1), 100);
		assert_ok!(Assets::freeze_asset(RuntimeOrigin::signed(1), 0));
		assert_noop!(Assets::transfer(RuntimeOrigin::signed(1), 0, 2, 50), Error::<Test>::Frozen);
		assert_ok!(Assets::thaw_asset(RuntimeOrigin::signed(1), 0));
		assert_ok!(Assets::transfer(RuntimeOrigin::signed(1), 0, 2, 50));
	});
}

#[test]
fn approve_transfer_frozen_asset_should_not_work() {
	new_test_ext().execute_with(|| {
		Balances::make_free_balance_be(&1, 100);
		assert_ok!(Assets::force_create(RuntimeOrigin::root(), 0, 1, true, 1));
		assert_ok!(Assets::mint(RuntimeOrigin::signed(1), 0, 1, 100));
		assert_eq!(Assets::balance(0, 1), 100);
		assert_ok!(Assets::freeze_asset(RuntimeOrigin::signed(1), 0));
		assert_noop!(
			Assets::approve_transfer(RuntimeOrigin::signed(1), 0, 2, 50),
			Error::<Test>::Frozen
		);
		assert_ok!(Assets::thaw_asset(RuntimeOrigin::signed(1), 0));
		assert_ok!(Assets::approve_transfer(RuntimeOrigin::signed(1), 0, 2, 50));
	});
}

#[test]
fn origin_guards_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::force_create(RuntimeOrigin::root(), 0, 1, true, 1));
		assert_ok!(Assets::mint(RuntimeOrigin::signed(1), 0, 1, 100));
		assert_noop!(
			Assets::transfer_ownership(RuntimeOrigin::signed(2), 0, 2),
			Error::<Test>::NoPermission
		);
		assert_noop!(
			Assets::set_team(RuntimeOrigin::signed(2), 0, 2, 2, 2),
			Error::<Test>::NoPermission
		);
		assert_noop!(Assets::freeze(RuntimeOrigin::signed(2), 0, 1), Error::<Test>::NoPermission);
		assert_noop!(Assets::thaw(RuntimeOrigin::signed(2), 0, 2), Error::<Test>::NoPermission);
		assert_noop!(
			Assets::mint(RuntimeOrigin::signed(2), 0, 2, 100),
			Error::<Test>::NoPermission
		);
		assert_noop!(
			Assets::burn(RuntimeOrigin::signed(2), 0, 1, 100),
			Error::<Test>::NoPermission
		);
		assert_noop!(
			Assets::force_transfer(RuntimeOrigin::signed(2), 0, 1, 2, 100),
			Error::<Test>::NoPermission
		);
		let w = Asset::<Test>::get(0).unwrap().destroy_witness();
		assert_noop!(Assets::destroy(RuntimeOrigin::signed(2), 0, w), Error::<Test>::NoPermission);
	});
}

#[test]
fn transfer_owner_should_work() {
	new_test_ext().execute_with(|| {
		Balances::make_free_balance_be(&1, 100);
		Balances::make_free_balance_be(&2, 100);
		assert_ok!(Assets::create(RuntimeOrigin::signed(1), 0, 1, 1));

		assert_eq!(Balances::reserved_balance(&1), 1);

		assert_ok!(Assets::transfer_ownership(RuntimeOrigin::signed(1), 0, 2));
		assert_eq!(Balances::reserved_balance(&2), 1);
		assert_eq!(Balances::reserved_balance(&1), 0);

		assert_noop!(
			Assets::transfer_ownership(RuntimeOrigin::signed(1), 0, 1),
			Error::<Test>::NoPermission
		);

		// Set metadata now and make sure that deposit gets transferred back.
		assert_ok!(Assets::set_metadata(
			RuntimeOrigin::signed(2),
			0,
			vec![0u8; 10],
			vec![0u8; 10],
			12
		));
		assert_ok!(Assets::transfer_ownership(RuntimeOrigin::signed(2), 0, 1));
		assert_eq!(Balances::reserved_balance(&1), 22);
		assert_eq!(Balances::reserved_balance(&2), 0);
	});
}

#[test]
fn set_team_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::force_create(RuntimeOrigin::root(), 0, 1, true, 1));
		assert_ok!(Assets::set_team(RuntimeOrigin::signed(1), 0, 2, 3, 4));

		assert_ok!(Assets::mint(RuntimeOrigin::signed(2), 0, 2, 100));
		assert_ok!(Assets::freeze(RuntimeOrigin::signed(4), 0, 2));
		assert_ok!(Assets::thaw(RuntimeOrigin::signed(3), 0, 2));
		assert_ok!(Assets::force_transfer(RuntimeOrigin::signed(3), 0, 2, 3, 100));
		assert_ok!(Assets::burn(RuntimeOrigin::signed(3), 0, 3, 100));
	});
}

#[test]
fn transferring_to_frozen_account_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::force_create(RuntimeOrigin::root(), 0, 1, true, 1));
		assert_ok!(Assets::mint(RuntimeOrigin::signed(1), 0, 1, 100));
		assert_ok!(Assets::mint(RuntimeOrigin::signed(1), 0, 2, 100));
		assert_eq!(Assets::balance(0, 1), 100);
		assert_eq!(Assets::balance(0, 2), 100);
		assert_ok!(Assets::freeze(RuntimeOrigin::signed(1), 0, 2));
		assert_ok!(Assets::transfer(RuntimeOrigin::signed(1), 0, 2, 50));
		assert_eq!(Assets::balance(0, 2), 150);
	});
}

#[test]
fn transferring_amount_more_than_available_balance_should_not_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::force_create(RuntimeOrigin::root(), 0, 1, true, 1));
		assert_ok!(Assets::mint(RuntimeOrigin::signed(1), 0, 1, 100));
		assert_eq!(Assets::balance(0, 1), 100);
		assert_ok!(Assets::transfer(RuntimeOrigin::signed(1), 0, 2, 50));
		assert_eq!(Assets::balance(0, 1), 50);
		assert_eq!(Assets::balance(0, 2), 50);
		assert_ok!(Assets::burn(RuntimeOrigin::signed(1), 0, 1, u128::MAX));
		assert_eq!(Assets::balance(0, 1), 0);
		assert_noop!(
			Assets::transfer(RuntimeOrigin::signed(1), 0, 1, 50),
			Error::<Test>::NoAccount
		);
		assert_noop!(
			Assets::transfer(RuntimeOrigin::signed(2), 0, 1, 51),
			Error::<Test>::BalanceLow
		);
	});
}

#[test]
fn transferring_less_than_one_unit_is_fine() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::force_create(RuntimeOrigin::root(), 0, 1, true, 1));
		assert_ok!(Assets::mint(RuntimeOrigin::signed(1), 0, 1, 100));
		assert_eq!(Assets::balance(0, 1), 100);
		assert_ok!(Assets::transfer(RuntimeOrigin::signed(1), 0, 2, 0));
		// `ForceCreated` and `Issued` but no `Transferred` event.
		assert_eq!(System::events().len(), 2);
	});
}

#[test]
fn transferring_more_units_than_total_supply_should_not_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::force_create(RuntimeOrigin::root(), 0, 1, true, 1));
		assert_ok!(Assets::mint(RuntimeOrigin::signed(1), 0, 1, 100));
		assert_eq!(Assets::balance(0, 1), 100);
		assert_noop!(
			Assets::transfer(RuntimeOrigin::signed(1), 0, 2, 101),
			Error::<Test>::BalanceLow
		);
	});
}

#[test]
fn burning_asset_balance_with_positive_balance_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::force_create(RuntimeOrigin::root(), 0, 1, true, 1));
		assert_ok!(Assets::mint(RuntimeOrigin::signed(1), 0, 1, 100));
		assert_eq!(Assets::balance(0, 1), 100);
		assert_ok!(Assets::burn(RuntimeOrigin::signed(1), 0, 1, u128::MAX));
		assert_eq!(Assets::balance(0, 1), 0);
	});
}

#[test]
fn burning_asset_balance_with_zero_balance_does_nothing() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::force_create(RuntimeOrigin::root(), 0, 1, true, 1));
		assert_ok!(Assets::mint(RuntimeOrigin::signed(1), 0, 1, 100));
		assert_eq!(Assets::balance(0, 2), 0);
		assert_noop!(
			Assets::burn(RuntimeOrigin::signed(1), 0, 2, u128::MAX),
			Error::<Test>::NoAccount
		);
		assert_eq!(Assets::balance(0, 2), 0);
		assert_eq!(Assets::total_supply(0), 100);
	});
}

#[test]
fn set_metadata_should_work() {
	new_test_ext().execute_with(|| {
		// Cannot add metadata to unknown asset
		assert_noop!(
			Assets::set_metadata(RuntimeOrigin::signed(1), 0, vec![0u8; 10], vec![0u8; 10], 12),
			Error::<Test>::Unknown,
		);
		assert_ok!(Assets::force_create(RuntimeOrigin::root(), 0, 1, true, 1));
		// Cannot add metadata to unowned asset
		assert_noop!(
			Assets::set_metadata(RuntimeOrigin::signed(2), 0, vec![0u8; 10], vec![0u8; 10], 12),
			Error::<Test>::NoPermission,
		);

		// Cannot add oversized metadata
		assert_noop!(
			Assets::set_metadata(RuntimeOrigin::signed(1), 0, vec![0u8; 100], vec![0u8; 10], 12),
			Error::<Test>::BadMetadata,
		);
		assert_noop!(
			Assets::set_metadata(RuntimeOrigin::signed(1), 0, vec![0u8; 10], vec![0u8; 100], 12),
			Error::<Test>::BadMetadata,
		);

		// Successfully add metadata and take deposit
		Balances::make_free_balance_be(&1, 30);
		assert_ok!(Assets::set_metadata(
			RuntimeOrigin::signed(1),
			0,
			vec![0u8; 10],
			vec![0u8; 10],
			12
		));
		assert_eq!(Balances::free_balance(&1), 9);

		// Update deposit
		assert_ok!(Assets::set_metadata(
			RuntimeOrigin::signed(1),
			0,
			vec![0u8; 10],
			vec![0u8; 5],
			12
		));
		assert_eq!(Balances::free_balance(&1), 14);
		assert_ok!(Assets::set_metadata(
			RuntimeOrigin::signed(1),
			0,
			vec![0u8; 10],
			vec![0u8; 15],
			12
		));
		assert_eq!(Balances::free_balance(&1), 4);

		// Cannot over-reserve
		assert_noop!(
			Assets::set_metadata(RuntimeOrigin::signed(1), 0, vec![0u8; 20], vec![0u8; 20], 12),
			BalancesError::<Test, _>::InsufficientBalance,
		);

		// Clear Metadata
		assert!(Metadata::<Test>::contains_key(0));
		assert_noop!(
			Assets::clear_metadata(RuntimeOrigin::signed(2), 0),
			Error::<Test>::NoPermission
		);
		assert_noop!(Assets::clear_metadata(RuntimeOrigin::signed(1), 1), Error::<Test>::Unknown);
		assert_ok!(Assets::clear_metadata(RuntimeOrigin::signed(1), 0));
		assert!(!Metadata::<Test>::contains_key(0));
	});
}

/// Destroying an asset calls the `FrozenBalance::died` hooks of all accounts.
#[test]
fn destroy_calls_died_hooks() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::force_create(RuntimeOrigin::root(), 0, 1, true, 50));
		// Create account 1 and 2.
		assert_ok!(Assets::mint(RuntimeOrigin::signed(1), 0, 1, 100));
		assert_ok!(Assets::mint(RuntimeOrigin::signed(1), 0, 2, 100));
		// Destroy the asset.
		let w = Asset::<Test>::get(0).unwrap().destroy_witness();
		assert_ok!(Assets::destroy(RuntimeOrigin::signed(1), 0, w));

		// Asset is gone and accounts 1 and 2 died.
		assert!(Asset::<Test>::get(0).is_none());
		assert_eq!(hooks(), vec![Hook::Died(0, 1), Hook::Died(0, 2)]);
	})
}

#[test]
fn freezer_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::force_create(RuntimeOrigin::root(), 0, 1, true, 10));
		assert_ok!(Assets::mint(RuntimeOrigin::signed(1), 0, 1, 100));
		assert_eq!(Assets::balance(0, 1), 100);

		// freeze 50 of it.
		set_frozen_balance(0, 1, 50);

		assert_ok!(Assets::transfer(RuntimeOrigin::signed(1), 0, 2, 20));
		// cannot transfer another 21 away as this would take the non-frozen balance (30) to below
		// the minimum balance (10).
		assert_noop!(
			Assets::transfer(RuntimeOrigin::signed(1), 0, 2, 21),
			Error::<Test>::BalanceLow
		);

		// create an approved transfer...
		Balances::make_free_balance_be(&1, 100);
		assert_ok!(Assets::approve_transfer(RuntimeOrigin::signed(1), 0, 2, 50));
		let e = Error::<Test>::BalanceLow;
		// ...but that wont work either:
		assert_noop!(Assets::transfer_approved(RuntimeOrigin::signed(2), 0, 1, 2, 21), e);
		// a force transfer won't work also.
		let e = Error::<Test>::BalanceLow;
		assert_noop!(Assets::force_transfer(RuntimeOrigin::signed(1), 0, 1, 2, 21), e);

		// reduce it to only 49 frozen...
		set_frozen_balance(0, 1, 49);
		// ...and it's all good:
		assert_ok!(Assets::force_transfer(RuntimeOrigin::signed(1), 0, 1, 2, 21));

		// and if we clear it, we can remove the account completely.
		clear_frozen_balance(0, 1);
		assert_ok!(Assets::transfer(RuntimeOrigin::signed(1), 0, 2, 50));
		assert_eq!(hooks(), vec![Hook::Died(0, 1)]);
	});
}

#[test]
fn imbalances_should_work() {
	use frame_support::traits::tokens::fungibles::Balanced;

	new_test_ext().execute_with(|| {
		assert_ok!(Assets::force_create(RuntimeOrigin::root(), 0, 1, true, 1));

		let imb = Assets::issue(0, 100);
		assert_eq!(Assets::total_supply(0), 100);
		assert_eq!(imb.peek(), 100);

		let (imb1, imb2) = imb.split(30);
		assert_eq!(imb1.peek(), 30);
		assert_eq!(imb2.peek(), 70);

		drop(imb2);
		assert_eq!(Assets::total_supply(0), 30);

		assert!(Assets::resolve(&1, imb1).is_ok());
		assert_eq!(Assets::balance(0, 1), 30);
		assert_eq!(Assets::total_supply(0), 30);
	});
}

#[test]
fn force_metadata_should_work() {
	new_test_ext().execute_with(|| {
		// force set metadata works
		assert_ok!(Assets::force_create(RuntimeOrigin::root(), 0, 1, true, 1));
		assert_ok!(Assets::force_set_metadata(
			RuntimeOrigin::root(),
			0,
			vec![0u8; 10],
			vec![0u8; 10],
			8,
			false
		));
		assert!(Metadata::<Test>::contains_key(0));

		// overwrites existing metadata
		let asset_original_metadata = Metadata::<Test>::get(0);
		assert_ok!(Assets::force_set_metadata(
			RuntimeOrigin::root(),
			0,
			vec![1u8; 10],
			vec![1u8; 10],
			8,
			false
		));
		assert_ne!(Metadata::<Test>::get(0), asset_original_metadata);

		// attempt to set metadata for non-existent asset class
		assert_noop!(
			Assets::force_set_metadata(
				RuntimeOrigin::root(),
				1,
				vec![0u8; 10],
				vec![0u8; 10],
				8,
				false
			),
			Error::<Test>::Unknown
		);

		// string length limit check
		let limit = 50usize;
		assert_noop!(
			Assets::force_set_metadata(
				RuntimeOrigin::root(),
				0,
				vec![0u8; limit + 1],
				vec![0u8; 10],
				8,
				false
			),
			Error::<Test>::BadMetadata
		);
		assert_noop!(
			Assets::force_set_metadata(
				RuntimeOrigin::root(),
				0,
				vec![0u8; 10],
				vec![0u8; limit + 1],
				8,
				false
			),
			Error::<Test>::BadMetadata
		);

		// force clear metadata works
		assert!(Metadata::<Test>::contains_key(0));
		assert_ok!(Assets::force_clear_metadata(RuntimeOrigin::root(), 0));
		assert!(!Metadata::<Test>::contains_key(0));

		// Error handles clearing non-existent asset class
		assert_noop!(
			Assets::force_clear_metadata(RuntimeOrigin::root(), 1),
			Error::<Test>::Unknown
		);
	});
}

#[test]
fn force_asset_status_should_work() {
	new_test_ext().execute_with(|| {
		Balances::make_free_balance_be(&1, 10);
		Balances::make_free_balance_be(&2, 10);
		assert_ok!(Assets::create(RuntimeOrigin::signed(1), 0, 1, 30));
		assert_ok!(Assets::mint(RuntimeOrigin::signed(1), 0, 1, 50));
		assert_ok!(Assets::mint(RuntimeOrigin::signed(1), 0, 2, 150));

		// force asset status to change min_balance > balance
		assert_ok!(Assets::force_asset_status(
			RuntimeOrigin::root(),
			0,
			1,
			1,
			1,
			1,
			100,
			true,
			false
		));
		assert_eq!(Assets::balance(0, 1), 50);

		// account can recieve assets for balance < min_balance
		assert_ok!(Assets::transfer(RuntimeOrigin::signed(2), 0, 1, 1));
		assert_eq!(Assets::balance(0, 1), 51);

		// account on outbound transfer will cleanup for balance < min_balance
		assert_ok!(Assets::transfer(RuntimeOrigin::signed(1), 0, 2, 1));
		assert_eq!(Assets::balance(0, 1), 0);

		// won't create new account with balance below min_balance
		assert_noop!(
			Assets::transfer(RuntimeOrigin::signed(2), 0, 3, 50),
			TokenError::BelowMinimum
		);

		// force asset status will not execute for non-existent class
		assert_noop!(
			Assets::force_asset_status(RuntimeOrigin::root(), 1, 1, 1, 1, 1, 90, true, false),
			Error::<Test>::Unknown
		);

		// account drains to completion when funds dip below min_balance
		assert_ok!(Assets::force_asset_status(
			RuntimeOrigin::root(),
			0,
			1,
			1,
			1,
			1,
			110,
			true,
			false
		));
		assert_ok!(Assets::transfer(RuntimeOrigin::signed(2), 0, 1, 110));
		assert_eq!(Assets::balance(0, 1), 200);
		assert_eq!(Assets::balance(0, 2), 0);
		assert_eq!(Assets::total_supply(0), 200);
	});
}

#[test]
fn balance_conversion_should_work() {
	new_test_ext().execute_with(|| {
		use frame_support::traits::tokens::BalanceConversion;

		let id = 42;
		assert_ok!(Assets::force_create(RuntimeOrigin::root(), id, 1, true, 10));
		let not_sufficient = 23;
		assert_ok!(Assets::force_create(RuntimeOrigin::root(), not_sufficient, 1, false, 10));

		assert_eq!(
			BalanceToAssetBalance::<Balances, Test, ConvertInto>::to_asset_balance(100, 1234),
			Err(ConversionError::AssetMissing)
		);
		assert_eq!(
			BalanceToAssetBalance::<Balances, Test, ConvertInto>::to_asset_balance(
				100,
				not_sufficient
			),
			Err(ConversionError::AssetNotSufficient)
		);
		// 10 / 1 == 10 -> the conversion should 10x the value
		assert_eq!(
			BalanceToAssetBalance::<Balances, Test, ConvertInto>::to_asset_balance(100, id),
			Ok(100 * 10)
		);
	});
}

#[test]
fn assets_from_genesis_should_exist() {
	new_test_ext().execute_with(|| {
		assert!(Asset::<Test>::contains_key(999));
		assert!(Metadata::<Test>::contains_key(999));
		assert_eq!(Assets::balance(999, 1), 100);
		assert_eq!(Assets::total_supply(999), 100);
	});
}

#[test]
fn querying_name_symbol_and_decimals_should_work() {
	new_test_ext().execute_with(|| {
		use frame_support::traits::tokens::fungibles::metadata::Inspect;
		assert_ok!(Assets::force_create(RuntimeOrigin::root(), 0, 1, true, 1));
		assert_ok!(Assets::force_set_metadata(
			RuntimeOrigin::root(),
			0,
			vec![0u8; 10],
			vec![1u8; 10],
			12,
			false
		));
		assert_eq!(Assets::name(0), vec![0u8; 10]);
		assert_eq!(Assets::symbol(0), vec![1u8; 10]);
		assert_eq!(Assets::decimals(0), 12);
	});
}

#[test]
fn querying_allowance_should_work() {
	new_test_ext().execute_with(|| {
		use frame_support::traits::tokens::fungibles::approvals::{Inspect, Mutate};
		assert_ok!(Assets::force_create(RuntimeOrigin::root(), 0, 1, true, 1));
		assert_ok!(Assets::mint(RuntimeOrigin::signed(1), 0, 1, 100));
		Balances::make_free_balance_be(&1, 1);
		assert_ok!(Assets::approve(0, &1, &2, 50));
		assert_eq!(Assets::allowance(0, &1, &2), 50);
		// Transfer asset 0, from owner 1 and delegate 2 to destination 3
		assert_ok!(Assets::transfer_from(0, &1, &2, &3, 50));
		assert_eq!(Assets::allowance(0, &1, &2), 0);
	});
}

#[test]
fn transfer_large_asset() {
	new_test_ext().execute_with(|| {
		let amount = u128::pow(2, 127) + 2;
		assert_ok!(Assets::force_create(RuntimeOrigin::root(), 0, 1, true, 1));
		assert_ok!(Assets::mint(RuntimeOrigin::signed(1), 0, 1, amount));
		assert_ok!(Assets::transfer(RuntimeOrigin::signed(1), 0, 2, amount - 1));
	})
}

#[test]
fn querying_roles_should_work() {
	new_test_ext().execute_with(|| {
		use frame_support::traits::tokens::fungibles::roles::Inspect;
		assert_ok!(Assets::force_create(RuntimeOrigin::root(), 0, 1, true, 1));
		assert_ok!(Assets::set_team(
			RuntimeOrigin::signed(1),
			0,
			// Issuer
			2,
			// Admin
			3,
			// Freezer
			4,
		));
		assert_eq!(Assets::owner(0), Some(1));
		assert_eq!(Assets::issuer(0), Some(2));
		assert_eq!(Assets::admin(0), Some(3));
		assert_eq!(Assets::freezer(0), Some(4));
	});
}

// tests of omniverse tokens
const CHAIN_ID: u8 = 1;
const TOKEN_ID: Vec<u8> = Vec::<u8>::new();

fn get_sig_slice(sig: &RecoverableSignature) -> [u8; 65] {
    let (recovery_id, sig_slice) = sig.serialize_compact();
    let mut sig_recovery: [u8; 65] = [0; 65];
    sig_recovery[0..64].copy_from_slice(&sig_slice);
    sig_recovery[64] = recovery_id.to_i32() as u8;
    sig_recovery
}

fn encode_transfer(secp: &Secp256k1<secp256k1::All>, from: (SecretKey, PublicKey),
    to: PublicKey, amount: u128, nonce: u128) -> OmniverseTokenProtocol {
    let pk_from: [u8; 64] = from.1.serialize_uncompressed()[1..].try_into().expect("");
    let pk_to: [u8; 64] = to.serialize_uncompressed()[1..].try_into().expect("");
    let transfer_data = TransferTokenOp::new(pk_to, amount).encode();
    let data = TokenOpcode::new(TRANSFER, transfer_data).encode();
    let mut tx_data = OmniverseTokenProtocol::new(nonce, CHAIN_ID, pk_from, TOKEN_ID, data);
    let h = tx_data.get_raw_hash();
    let message = Message::from_slice(h.as_slice()).expect("messages must be 32 bytes and are expected to be hashes");
    let sig: RecoverableSignature = secp.sign_ecdsa_recoverable(&message, &from.0);
    let sig_recovery = get_sig_slice(&sig);
    tx_data.set_signature(sig_recovery);
    tx_data
}

fn encode_mint(secp: &Secp256k1<secp256k1::All>, from: (SecretKey, PublicKey),
    to: PublicKey, amount: u128, nonce: u128) -> OmniverseTokenProtocol {
    let pk_from: [u8; 64] = from.1.serialize_uncompressed()[1..].try_into().expect("");
    let pk_to: [u8; 64] = to.serialize_uncompressed()[1..].try_into().expect("");
    let transfer_data = MintTokenOp::new(pk_to, amount).encode();
    let data = TokenOpcode::new(MINT, transfer_data).encode();
    let mut tx_data = OmniverseTokenProtocol::new(nonce, CHAIN_ID, pk_from, TOKEN_ID, data);
    let h = tx_data.get_raw_hash();
    let message = Message::from_slice(h.as_slice()).expect("messages must be 32 bytes and are expected to be hashes");
    let sig: RecoverableSignature = secp.sign_ecdsa_recoverable(&message, &from.0);
    let sig_recovery = get_sig_slice(&sig);
    tx_data.set_signature(sig_recovery);
    tx_data
}

// #[test]
// fn it_works_for_decode() {
//     new_test_ext().execute_with(|| {
//         let data = [
//             3,  65,   1, 123, 189, 136, 115, 207, 195,  13,  61, 222,
//           226, 167, 169, 220, 210, 181, 179, 153, 184,  93, 171, 135,
//           192,  17, 173,  75, 233, 111, 230, 150,  37,  67,  14,  63,
//            19, 148, 114,   7, 255,  78,  89,  91,  67, 238, 127,  43,
//           205, 103, 208, 179,  37,  39,  55,  40, 111, 234, 152, 103,
//           135, 234,  57, 187, 219, 106, 181, 100,   0,   0,   0,   0,
//             0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0
//         ];

//         let token_op = TokenOpcode::decode(&mut data.as_slice()).unwrap();
//         println!("{:?}", token_op);
//         let mint_op = MintTokenOp::decode(&mut token_op.data.as_slice());
//         println!("{:?}", mint_op);
//     });
// }

#[test]
fn it_works_for_create_token() {
    new_test_ext().execute_with(|| {
        assert_ok!(Assets::create_token(RuntimeOrigin::signed(1), [0; 64], vec![1], None));
        assert!(Assets::tokens_info(vec![1]).is_some());
    });
}

#[test]
fn it_fails_for_create_token_with_token_already_exist() {
    new_test_ext().execute_with(|| {
        assert_ok!(Assets::create_token(RuntimeOrigin::signed(1), [0; 64], vec![1], None));
        assert_err!(Assets::create_token(RuntimeOrigin::signed(1), [0; 64], vec![1], None), Error::<Test>::InUse);
    });
}

#[test]
fn it_fails_for_set_members_with_token_not_exist() {
    new_test_ext().execute_with(|| {
        assert_err!(Assets::set_members(RuntimeOrigin::signed(1), vec![1], vec![1]), Error::<Test>::Unknown);
    });
}

#[test]
fn it_fails_for_set_members_with_not_owner() {
    new_test_ext().execute_with(|| {
        assert_ok!(Assets::create_token(RuntimeOrigin::signed(1), [0; 64], vec![1], None));
        assert_err!(Assets::set_members(RuntimeOrigin::signed(2), vec![1], vec![1]), Error::<Test>::SignerNotOwner);
    });
}

#[test]
fn it_works_for_set_members() {
    new_test_ext().execute_with(|| {
        assert_ok!(Assets::create_token(RuntimeOrigin::signed(1), [0; 64], vec![1], None));
        assert_ok!(Assets::set_members(RuntimeOrigin::signed(1), vec![1], vec![1]));
        let token_info = Assets::tokens_info(vec![1]).unwrap();
        assert!(token_info.members == vec![1]);
    });
}

#[test]
fn it_fails_for_factory_handler_with_token_not_exist() {
    new_test_ext().execute_with(|| {
        let secp = Secp256k1::new();
        // Generate key pair
        let (secret_key, public_key) = secp.generate_keypair(&mut OsRng);

        // Get nonce
        let pk: [u8; 64] = public_key.serialize_uncompressed()[1..].try_into().expect("");
        let nonce = OmniverseProtocol::get_transaction_count(pk);

        let data = encode_transfer(&secp, (secret_key, public_key), public_key, 1, nonce);
        assert_err!(Assets::send_transaction_external(vec![1], &data), Error::<Test>::Unknown);
    });
}

#[test]
fn it_fails_for_factory_handler_with_wrong_destination() {
    new_test_ext().execute_with(|| {
        let secp = Secp256k1::new();
        // Generate key pair
        let (secret_key, public_key) = secp.generate_keypair(&mut OsRng);

        // Get nonce
        let pk: [u8; 64] = public_key.serialize_uncompressed()[1..].try_into().expect("");
        let nonce = OmniverseProtocol::get_transaction_count(pk);
        
        // Create token
        assert_ok!(Assets::create_token(RuntimeOrigin::signed(1), pk, vec![1], None));

        let (_, public_key_to) = secp.generate_keypair(&mut OsRng);
        let data = encode_transfer(&secp, (secret_key, public_key), public_key_to, 1, nonce);
        assert_err!(Assets::send_transaction_external(vec![1], &data), Error::<Test>::WrongDestination);
    });
}

#[test]
fn it_fails_for_factory_handler_with_signature_error() {
    new_test_ext().execute_with(|| {
        let secp = Secp256k1::new();
        // Generate key pair
        let (secret_key, public_key) = secp.generate_keypair(&mut OsRng);

        // Get nonce
        let pk: [u8; 64] = public_key.serialize_uncompressed()[1..].try_into().expect("");
        let nonce = OmniverseProtocol::get_transaction_count(pk);
        
        // Create token
        assert_ok!(Assets::create_token(RuntimeOrigin::signed(1), pk, TOKEN_ID, None));

        let (_, public_key_to) = secp.generate_keypair(&mut OsRng);
        let mut data = encode_transfer(&secp, (secret_key, public_key), public_key_to, 1, nonce);
        data.signature = [0; 65];
        assert_err!(Assets::send_transaction_external(TOKEN_ID, &data), Error::<Test>::ProtocolSignatureError);
    });
}

#[test]
fn it_fails_for_factory_handler_mint_with_signer_not_owner() {
    new_test_ext().execute_with(|| {
        let secp = Secp256k1::new();
        // Generate key pair
        let (_, public_key) = secp.generate_keypair(&mut OsRng);

        // Get nonce
        let pk: [u8; 64] = public_key.serialize_uncompressed()[1..].try_into().expect("");
        let nonce = OmniverseProtocol::get_transaction_count(pk);
        
        // Create token
        assert_ok!(Assets::create_token(RuntimeOrigin::signed(1), pk, TOKEN_ID, None));

        let (secret_key_to, public_key_to) = secp.generate_keypair(&mut OsRng);
        let data = encode_mint(&secp, (secret_key_to, public_key_to), public_key_to, 1, nonce);
        assert_err!(Assets::send_transaction_external(TOKEN_ID, &data), Error::<Test>::SignerNotOwner);
    });
}

#[test]
fn it_works_for_factory_handler_mint() {
    new_test_ext().execute_with(|| {
        let secp = Secp256k1::new();
        // Generate key pair
        let (secret_key, public_key) = secp.generate_keypair(&mut OsRng);

        // Get nonce
        let pk: [u8; 64] = public_key.serialize_uncompressed()[1..].try_into().expect("");
        let nonce = OmniverseProtocol::get_transaction_count(pk);
        
        // Create token
        assert_ok!(Assets::create_token(RuntimeOrigin::signed(1), pk, TOKEN_ID, None));

        let (_, public_key_to) = secp.generate_keypair(&mut OsRng);
        let data = encode_mint(&secp, (secret_key, public_key), public_key_to, 1, nonce);
        assert_ok!(Assets::send_transaction_external(TOKEN_ID, &data));

        let pk_to: [u8; 64] = public_key_to.serialize_uncompressed()[1..].try_into().expect("");
        let token = Assets::tokens(TOKEN_ID, pk_to);
        assert_eq!(token, 1);
    });
}

#[test]
fn it_fails_for_factory_handler_transfer_with_balance_overflow() {
    new_test_ext().execute_with(|| {
        let secp = Secp256k1::new();
        // Generate key pair
        let (secret_key, public_key) = secp.generate_keypair(&mut OsRng);

        // Get nonce
        let pk: [u8; 64] = public_key.serialize_uncompressed()[1..].try_into().expect("");
        let nonce = OmniverseProtocol::get_transaction_count(pk);

        // Create token
        assert_ok!(Assets::create_token(RuntimeOrigin::signed(1), pk, TOKEN_ID, None));

        let (_, public_key_to) = secp.generate_keypair(&mut OsRng);
        let data = encode_transfer(&secp, (secret_key, public_key), public_key_to, 1, nonce);
        assert_err!(Assets::send_transaction_external(TOKEN_ID, &data), Error::<Test>::BalanceLow);
    });
}

#[test]
fn it_works_for_factory_handler_transfer() {
    new_test_ext().execute_with(|| {
        let secp = Secp256k1::new();
        // Generate key pair
        let (secret_key, public_key) = secp.generate_keypair(&mut OsRng);

        // Get nonce
        let pk: [u8; 64] = public_key.serialize_uncompressed()[1..].try_into().expect("");
        let nonce = OmniverseProtocol::get_transaction_count(pk);

        // Create token
        assert_ok!(Assets::create_token(RuntimeOrigin::signed(1), pk, TOKEN_ID, None));

        // Mint token
        let mint_data = encode_mint(&secp, (secret_key, public_key), public_key, 10, nonce);
        assert_ok!(Assets::send_transaction_external(TOKEN_ID, &mint_data));

        let (_, public_key_to) = secp.generate_keypair(&mut OsRng);
        let data = encode_transfer(&secp, (secret_key, public_key), public_key_to, 1, nonce);
        assert_ok!(Assets::send_transaction_external(TOKEN_ID, &data));

        assert_eq!(Assets::tokens(TOKEN_ID, &pk), 9);
        let pk_to: [u8; 64] = public_key_to.serialize_uncompressed()[1..].try_into().expect("");
        assert_eq!(Assets::tokens(TOKEN_ID, &pk_to), 1);
    });
}