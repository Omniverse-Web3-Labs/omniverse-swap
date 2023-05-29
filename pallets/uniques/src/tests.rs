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

//! Tests for Uniques pallet.

use crate::{mock::*, *};
use frame_support::{assert_err, assert_ok, traits::Currency};
use pallet_omniverse_protocol::OmniverseTx;
use pallet_omniverse_protocol::{
	traits::OmniverseAccounts, Fungible, OmniverseTransactionData, MINT, TRANSFER,
};
use secp256k1::rand::rngs::OsRng;
use secp256k1::{ecdsa::RecoverableSignature, Message, PublicKey, Secp256k1, SecretKey};
use sp_core::Hasher;
use sp_runtime::traits::BlakeTwo256;
use sp_std::prelude::*;

fn items() -> Vec<(u64, u32, u32)> {
	let mut r: Vec<_> = Account::<Test>::iter().map(|x| x.0).collect();
	r.sort();
	let mut s: Vec<_> = Item::<Test>::iter().map(|x| (x.2.owner, x.0, x.1)).collect();
	s.sort();
	assert_eq!(r, s);
	for collection in Item::<Test>::iter()
		.map(|x| x.0)
		.scan(None, |s, item| {
			if s.map_or(false, |last| last == item) {
				*s = Some(item);
				Some(None)
			} else {
				Some(Some(item))
			}
		})
		.flatten()
	{
		let details = Collection::<Test>::get(collection).unwrap();
		let items = Item::<Test>::iter_prefix(collection).count() as u32;
		assert_eq!(details.items, items);
	}
	r
}

#[test]
fn basic_setup_works() {
	new_test_ext().execute_with(|| {
		assert_eq!(items(), vec![]);
	});
}

// tests of omniverse tokens
const CHAIN_ID: u32 = 1;
const TOKEN_ID: Vec<u8> = Vec::<u8>::new();
const INITIATOR_ADDRESS: Vec<u8> = Vec::<u8>::new();
const COOL_DOWN: u64 = 10;

fn get_account_id_from_pk(pk: &[u8]) -> <Test as frame_system::Config>::AccountId {
	let hash = BlakeTwo256::hash(pk);
	let dest = <Test as frame_system::Config>::AccountId::decode(&mut &hash[..]).unwrap();
	dest
}

fn fund_account(account: <Test as frame_system::Config>::AccountId) {
	Balances::make_free_balance_be(&account, 1000);
}

fn get_sig_slice(sig: &RecoverableSignature) -> [u8; 65] {
	let (recovery_id, sig_slice) = sig.serialize_compact();
	let mut sig_recovery: [u8; 65] = [0; 65];
	sig_recovery[0..64].copy_from_slice(&sig_slice);
	sig_recovery[64] = recovery_id.to_i32() as u8;
	sig_recovery
}

fn encode_transfer(
	secp: &Secp256k1<secp256k1::All>,
	from: (SecretKey, PublicKey),
	to: PublicKey,
	amount: u128,
	nonce: u128,
) -> OmniverseTransactionData {
	let pk_from: [u8; 64] = from.1.serialize_uncompressed()[1..].try_into().expect("");
	let pk_to: [u8; 64] = to.serialize_uncompressed()[1..].try_into().expect("");
	let payload = Fungible::new(TRANSFER, pk_to.into(), amount).encode();
	let mut tx_data =
		OmniverseTransactionData::new(nonce, CHAIN_ID, INITIATOR_ADDRESS, pk_from, payload);
	let h = tx_data.get_raw_hash(false);
	let message = Message::from_slice(h.as_slice())
		.expect("messages must be 32 bytes and are expected to be hashes");
	let sig: RecoverableSignature = secp.sign_ecdsa_recoverable(&message, &from.0);
	let sig_recovery = get_sig_slice(&sig);
	tx_data.set_signature(sig_recovery);
	tx_data
}

fn encode_mint(
	secp: &Secp256k1<secp256k1::All>,
	from: (SecretKey, PublicKey),
	to: PublicKey,
	amount: u128,
	nonce: u128,
) -> OmniverseTransactionData {
	let pk_from: [u8; 64] = from.1.serialize_uncompressed()[1..].try_into().expect("");
	let pk_to: [u8; 64] = to.serialize_uncompressed()[1..].try_into().expect("");
	let payload = Fungible::new(MINT, pk_to.into(), amount).encode();
	let mut tx_data = OmniverseTransactionData::new(nonce, CHAIN_ID, TOKEN_ID, pk_from, payload);
	let h = tx_data.get_raw_hash(false);
	let message = Message::from_slice(h.as_slice())
		.expect("messages must be 32 bytes and are expected to be hashes");
	let sig: RecoverableSignature = secp.sign_ecdsa_recoverable(&message, &from.0);
	let sig_recovery = get_sig_slice(&sig);
	tx_data.set_signature(sig_recovery);
	tx_data
}

#[test]
fn create_token_should_work() {
	new_test_ext().execute_with(|| {
		let secp = Secp256k1::new();
		// Generate key pair
		let (_, public_key) = secp.generate_keypair(&mut OsRng);
		let pk: [u8; 64] = public_key.serialize_uncompressed()[1..].try_into().expect("");

		let account = get_account_id_from_pk(public_key.serialize().as_slice());
		fund_account(account);
		assert_ok!(Uniques::create_token(RuntimeOrigin::signed(1), pk, vec![1], None, None));
		assert!(Uniques::tokens_info(vec![1]).is_some());
	});
}

#[test]
fn create_token_with_token_already_exist_not_work() {
	new_test_ext().execute_with(|| {
		let secp = Secp256k1::new();
		// Generate key pair
		let (_, public_key) = secp.generate_keypair(&mut OsRng);
		let pk: [u8; 64] = public_key.serialize_uncompressed()[1..].try_into().expect("");

		let account = get_account_id_from_pk(public_key.serialize().as_slice());
		fund_account(account);
		assert_ok!(Uniques::create_token(
			RuntimeOrigin::signed(1),
			pk.clone(),
			vec![1],
			None,
			None
		));
		assert_err!(
			Uniques::create_token(RuntimeOrigin::signed(1), pk, vec![1], None, None),
			Error::<Test>::InUse
		);
	});
}

#[test]
fn set_members_with_token_not_exist_not_work() {
	new_test_ext().execute_with(|| {
		assert_err!(
			Uniques::set_members(RuntimeOrigin::signed(1), vec![1], vec![(1, Vec::new())]),
			Error::<Test>::UnknownCollection
		);
	});
}

#[test]
fn transfer_item_not_exist_not_work() {
	new_test_ext().execute_with(|| {
		let secp = Secp256k1::new();
		// Generate key pair
		let (secret_key, public_key) = secp.generate_keypair(&mut OsRng);

		// Get nonce
		let pk: [u8; 64] = public_key.serialize_uncompressed()[1..].try_into().expect("");
		let nonce = OmniverseProtocol::get_transaction_count(pk, PALLET_NAME.to_vec(), Vec::new());

		let data = encode_transfer(&secp, (secret_key, public_key), public_key, 1, nonce);
		assert_err!(
			Uniques::send_transaction_external(vec![1], &data),
			Error::<Test>::UnknownCollection
		);
	});
}

#[test]
fn mint_item_with_wrong_signature_not_work() {
	new_test_ext().execute_with(|| {
		let secp = Secp256k1::new();
		// Generate key pair
		let (secret_key, public_key) = secp.generate_keypair(&mut OsRng);

		// Get nonce
		let pk: [u8; 64] = public_key.serialize_uncompressed()[1..].try_into().expect("");
		let nonce = OmniverseProtocol::get_transaction_count(pk, PALLET_NAME.to_vec(), Vec::new());

		// Create token
		let account = get_account_id_from_pk(public_key.serialize().as_slice());
		fund_account(account);
		assert_ok!(Uniques::create_token(
			RuntimeOrigin::signed(1),
			pk,
			TOKEN_ID,
			Some(Vec::<(u32, Vec<u8>)>::new()),
			None
		));

		let (_, public_key_to) = secp.generate_keypair(&mut OsRng);
		let to = get_account_id_from_pk(public_key.serialize().as_slice());
		fund_account(to);

		// Mint token
		let mint_data = encode_mint(&secp, (secret_key, public_key), public_key, 100, nonce);
		assert_ok!(Uniques::send_transaction_external(TOKEN_ID, &mint_data));

		OmniverseProtocol::set_transaction_data(Some(OmniverseTx::new(
			mint_data,
			Timestamp::now().as_secs(),
		)));

		// Delay
		Timestamp::past(COOL_DOWN);
		assert_ok!(Uniques::trigger_execution(RuntimeOrigin::signed(1)));

		let mut data = encode_transfer(&secp, (secret_key, public_key), public_key_to, 1, nonce);
		data.signature = [0; 65];
		assert_err!(
			Uniques::send_transaction_external(TOKEN_ID, &data),
			Error::<Test>::ProtocolSignatureError
		);
	});
}

#[test]
fn not_owner_mint_item_with_not_work() {
	new_test_ext().execute_with(|| {
		let secp = Secp256k1::new();
		// Generate key pair
		let (_, public_key) = secp.generate_keypair(&mut OsRng);

		// Get nonce
		let pk: [u8; 64] = public_key.serialize_uncompressed()[1..].try_into().expect("");
		let nonce = OmniverseProtocol::get_transaction_count(pk, PALLET_NAME.to_vec(), Vec::new());

		// Create token
		let account = get_account_id_from_pk(public_key.serialize().as_slice());
		fund_account(account);
		assert_ok!(Uniques::create_token(
			RuntimeOrigin::signed(1),
			pk,
			TOKEN_ID,
			Some(Vec::<(u32, Vec<u8>)>::new()),
			None
		));

		let (secret_key_to, public_key_to) = secp.generate_keypair(&mut OsRng);
		let to = get_account_id_from_pk(public_key_to.serialize().as_slice());
		fund_account(to);

		let data = encode_mint(&secp, (secret_key_to, public_key_to), public_key_to, 1, nonce);
		assert_err!(
			Uniques::send_transaction_external(TOKEN_ID, &data),
			Error::<Test>::NoPermission
		);
	});
}

#[test]
fn mint_item_should_work() {
	new_test_ext().execute_with(|| {
		let secp = Secp256k1::new();
		// Generate key pair
		let (secret_key, public_key) = secp.generate_keypair(&mut OsRng);

		// Get nonce
		let pk: [u8; 64] = public_key.serialize_uncompressed()[1..].try_into().expect("");
		let nonce = OmniverseProtocol::get_transaction_count(pk, PALLET_NAME.to_vec(), Vec::new());

		// Create token
		let account = get_account_id_from_pk(public_key.serialize().as_slice());
		fund_account(account);
		assert_ok!(Uniques::create_token(
			RuntimeOrigin::signed(1),
			pk,
			TOKEN_ID,
			Some(Vec::<(u32, Vec<u8>)>::new()),
			None
		));

		let (_, public_key_to) = secp.generate_keypair(&mut OsRng);
		let account_to = get_account_id_from_pk(public_key_to.serialize().as_slice());
		fund_account(account_to);
		let data = encode_mint(&secp, (secret_key, public_key), public_key_to, 1, nonce);
		assert_ok!(Uniques::send_transaction_external(TOKEN_ID, &data));

		OmniverseProtocol::set_transaction_data(Some(OmniverseTx::new(
			data,
			Timestamp::now().as_secs(),
		)));

		// Delay
		Timestamp::past(COOL_DOWN);
		assert_ok!(Uniques::trigger_execution(RuntimeOrigin::signed(1)));

		let pk_to: [u8; 64] = public_key_to.serialize_uncompressed()[1..].try_into().expect("");
		let token = Uniques::tokens(TOKEN_ID, pk_to);
		assert_eq!(token, Some(vec![1]));
	});
}

#[test]
fn not_item_owner_transfer_should_not_work() {
	new_test_ext().execute_with(|| {
		let secp = Secp256k1::new();
		// Generate key pair
		let (secret_key, public_key) = secp.generate_keypair(&mut OsRng);

		// Get nonce
		let pk: [u8; 64] = public_key.serialize_uncompressed()[1..].try_into().expect("");
		let nonce = OmniverseProtocol::get_transaction_count(pk, PALLET_NAME.to_vec(), Vec::new());

		// Create token
		let account = get_account_id_from_pk(public_key.serialize().as_slice());
		fund_account(account);

		assert_ok!(Uniques::create_token(
			RuntimeOrigin::signed(1),
			pk,
			TOKEN_ID,
			Some(Vec::<(u32, Vec<u8>)>::new()),
			None
		));

		let (_, public_key_to) = secp.generate_keypair(&mut OsRng);
		let to = get_account_id_from_pk(public_key_to.serialize().as_slice());
		fund_account(to);

		// Mint token
		let mint_data = encode_mint(&secp, (secret_key, public_key), public_key, 1, nonce);
		assert_ok!(Uniques::send_transaction_external(TOKEN_ID, &mint_data));

		OmniverseProtocol::set_transaction_data(Some(OmniverseTx::new(
			mint_data,
			Timestamp::now().as_secs(),
		)));

		// Delay
		Timestamp::past(COOL_DOWN);
		assert_ok!(Uniques::trigger_execution(RuntimeOrigin::signed(1)));

		let data = encode_transfer(&secp, (secret_key, public_key), public_key_to, 10, nonce);
		assert_err!(
			Uniques::send_transaction_external(TOKEN_ID, &data),
			Error::<Test>::UnknownCollection
		);
	});
}

#[test]
fn transfer_item_should_work() {
	new_test_ext().execute_with(|| {
		let secp = Secp256k1::new();
		// Generate key pair
		let (secret_key, public_key) = secp.generate_keypair(&mut OsRng);

		// Get nonce
		let pk: [u8; 64] = public_key.serialize_uncompressed()[1..].try_into().expect("");
		let nonce = OmniverseProtocol::get_transaction_count(pk, PALLET_NAME.to_vec(), Vec::new());

		// Create token
		let account = get_account_id_from_pk(public_key.serialize().as_slice());
		fund_account(account);
		assert_ok!(Uniques::create_token(
			RuntimeOrigin::signed(1),
			pk,
			TOKEN_ID,
			Some(Vec::<(u32, Vec<u8>)>::new()),
			None
		));

		// Mint token
		let mint_data = encode_mint(&secp, (secret_key, public_key), public_key, 1, nonce);
		assert_ok!(Uniques::send_transaction_external(TOKEN_ID, &mint_data));

		OmniverseProtocol::set_transaction_data(Some(OmniverseTx::new(
			mint_data,
			Timestamp::now().as_secs(),
		)));

		// Delay
		Timestamp::past(COOL_DOWN);
		assert_ok!(Uniques::trigger_execution(RuntimeOrigin::signed(1)));

		let (_, public_key_to) = secp.generate_keypair(&mut OsRng);
		let account_to = get_account_id_from_pk(public_key_to.serialize().as_slice());
		fund_account(account_to);
		let data = encode_transfer(&secp, (secret_key, public_key), public_key_to, 1, nonce);
		assert_ok!(Uniques::send_transaction_external(TOKEN_ID, &data));

		OmniverseProtocol::set_transaction_data(Some(OmniverseTx::new(
			data,
			Timestamp::now().as_secs(),
		)));

		// Delay
		Timestamp::past(COOL_DOWN);
		assert_ok!(Uniques::trigger_execution(RuntimeOrigin::signed(1)));

		assert_eq!(Uniques::tokens(TOKEN_ID, &pk), Some(vec![]));
		let pk_to: [u8; 64] = public_key_to.serialize_uncompressed()[1..].try_into().expect("");
		assert_eq!(Uniques::tokens(TOKEN_ID, &pk_to), Some(vec![1]));
	});
}
