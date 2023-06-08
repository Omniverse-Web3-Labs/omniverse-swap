use crate::mock::*;
use codec::{Decode, Encode};
// use frame_support::assert_ok;
use frame_support::{assert_ok, traits::UnixTime};
use pallet_omniverse_protocol::{Fungible, OmniverseTransactionData, OmniverseTx, MINT, TRANSFER};
use secp256k1::rand::rngs::OsRng;
use secp256k1::rand::RngCore;
use secp256k1::{ecdsa::RecoverableSignature, Message, PublicKey, Secp256k1, SecretKey};
use sp_core::Hasher;
use sp_runtime::traits::BlakeTwo256;

const CHAIN_ID: u32 = 1;
static SECRET_KEY: [u8; 32] = [
	142, 190, 93, 31, 248, 244, 136, 11, 92, 255, 107, 224, 114, 68, 27, 173, 40, 169, 129, 200,
	15, 254, 183, 85, 187, 160, 40, 250, 193, 56, 118, 245,
];
// const TOKEN_ID: Vec<u8> = Vec::<u8>::new();

fn get_account_id_from_pk(pk: &[u8]) -> <Test as frame_system::Config>::AccountId {
	let hash = BlakeTwo256::hash(pk);
	let dest = <Test as frame_system::Config>::AccountId::decode(&mut &hash[..]).unwrap();
	dest
}

fn fund_account(account: <Test as frame_system::Config>::AccountId) {
	assert_ok!(Balances::transfer(RuntimeOrigin::signed(1), account, 50));
}

fn get_sig_slice(sig: &RecoverableSignature) -> [u8; 65] {
	let (recovery_id, sig_slice) = sig.serialize_compact();
	let mut sig_recovery: [u8; 65] = [0; 65];
	sig_recovery[0..64].copy_from_slice(&sig_slice);
	sig_recovery[64] = recovery_id.to_i32() as u8;
	sig_recovery
}

fn to_public_key(omniverse_account: &[u8; 64]) -> PublicKey {
	let mut vec = omniverse_account.to_vec();
	vec.insert(0, 4);
	PublicKey::from_slice(&vec).unwrap()
}

fn mint(
	secp: &Secp256k1<secp256k1::All>,
	token_id: &Vec<u8>,
	from: &(SecretKey, PublicKey),
	to: &[u8; 64],
	amount: u128,
	nonce: u128,
) {
	let pk_from: [u8; 64] = from.1.serialize_uncompressed()[1..].try_into().expect("");
	let payload = Fungible::new(MINT, to.to_vec(), amount).encode();
	let mut tx_data =
		OmniverseTransactionData::new(nonce, CHAIN_ID, token_id.clone(), pk_from, payload);
	let h = tx_data.get_raw_hash(false);
	let message = Message::from_slice(h.as_slice())
		.expect("messages must be 32 bytes and are expected to be hashes");
	let sig: RecoverableSignature = secp.sign_ecdsa_recoverable(&message, &from.0);
	let sig_recovery = get_sig_slice(&sig);
	tx_data.set_signature(sig_recovery);

	// send and execute transaction
	assert_ok!(Assets::send_transaction(
		RuntimeOrigin::signed(1),
		token_id.to_vec(),
		tx_data.clone()
	));
	OmniverseProtocol::set_transaction_data(Some(OmniverseTx::new(
		tx_data,
		Timestamp::now().as_secs(),
	)));
	assert_ok!(Assets::trigger_execution(RuntimeOrigin::signed(1)));
}

fn encode_transfer(
	secp: &Secp256k1<secp256k1::All>,
	token_id: &Vec<u8>,
	from: &(SecretKey, PublicKey),
	to: &[u8; 64],
	amount: u128,
	nonce: u128,
) -> OmniverseTransactionData {
	let pk_from: [u8; 64] = from.1.serialize_uncompressed()[1..].try_into().expect("");
	// let op_data = TransferTokenOp::new(pk_to, amount).encode();
	let payload = Fungible::new(TRANSFER, to.to_vec(), amount).encode();
	// let data = TokenOpcode::new(TRANSFER, transfer_data).encode();
	let mut tx_data =
		OmniverseTransactionData::new(nonce, CHAIN_ID, token_id.clone(), pk_from, payload);
	let h = tx_data.get_raw_hash(false);
	let message = Message::from_slice(h.as_slice())
		.expect("messages must be 32 bytes and are expected to be hashes");
	let sig: RecoverableSignature = secp.sign_ecdsa_recoverable(&message, &from.0);
	let sig_recovery = get_sig_slice(&sig);
	tx_data.set_signature(sig_recovery);
	tx_data
}

fn deposit(
	secp: &Secp256k1<secp256k1::All>,
	token_id: &Vec<u8>,
	from: &(SecretKey, PublicKey),
	amount: u128,
	nonce: u128,
) {
	let mpc = OmniSwap::mpc();
	let mpc_pk = to_public_key(&mpc);
	let account = get_account_id_from_pk(mpc_pk.serialize().as_slice());
	if Balances::free_balance(account) < 10 {
		fund_account(account);
	}
	let transfer_data = encode_transfer(&secp, token_id, from, &mpc, amount, nonce);

	assert_ok!(OmniSwap::deposit(
		RuntimeOrigin::signed(1),
		token_id.clone(),
		transfer_data.clone()
	));
	assert_ok!(Assets::trigger_execution(RuntimeOrigin::signed(1)));
}

// #[test]
// fn it_works_for_deposit() {
// 	let mut ext = new_test_ext();
// 	ext.execute_with(|| {
// 		let secp = Secp256k1::new();
// 		let secret_key = SecretKey::from_slice(&SECRET_KEY).unwrap();
// 		let public_key = PublicKey::from_secret_key(&secp, &secret_key);
// 		let mut token_id = [0u8; 32];
// 		OsRng.fill_bytes(&mut token_id);

// 		let pk: [u8; 64] = public_key.serialize_uncompressed()[1..].try_into().expect("");
// 		let account = get_account_id_from_pk(public_key.serialize().as_slice());
// 		fund_account(account);
// 		let amount = 10u128;
// 		let mut nonce = 0u128;
// 		// Create token
// 		assert_ok!(Assets::create_token(
// 			RuntimeOrigin::signed(1),
// 			pk.clone(),
// 			token_id.to_vec(),
// 			Some(Vec::<(u32, Vec<u8>)>::new()),
// 			None
// 		));

// 		// Mint token
// 		mint(&secp, &token_id.to_vec(), &(secret_key, public_key), &pk, amount, nonce);
// 		let pk_amount = Assets::tokens(token_id.to_vec(), pk);
// 		assert_eq!(pk_amount, amount);
// 		nonce += 1;
// 		deposit(&secp, &token_id.to_vec(), &(secret_key, public_key), amount, nonce);
// 		let mpc = OmniSwap::mpc();
// 		let mpc_amount = Assets::tokens(token_id.to_vec(), mpc);
// 		let pk_amount = Assets::tokens(token_id.to_vec(), pk);
// 		assert_eq!(mpc_amount, amount);
// 		assert_eq!(pk_amount, 0);
// 		// uncomfirm the deposit
// 		let balance = OmniSwap::balance(pk, token_id.to_vec()).unwrap_or(0);
// 		assert_eq!(balance, 0);

// 		// comfirm the deposit
// 		assert_ok!(OmniSwap::deposit_comfirm(RuntimeOrigin::signed(1), pk, token_id.into(), nonce));
// 		let balance = OmniSwap::balance(pk, token_id.to_vec()).unwrap_or(0);
// 		assert_eq!(balance, amount);

// 	});
// 	ext.as_backend();
// }

// #[test]
// fn it_works_for_add_liquidity() {
// 	new_test_ext().execute_with(|| {
// 		let secp = Secp256k1::new();
// 		// Generate key pair
// 		// let (secret_key, public_key) = secp.generate_keypair(&mut OsRng);
// 		let secret_key = SecretKey::from_slice(&SECRET_KEY).unwrap();
// 		let public_key = PublicKey::from_secret_key(&secp, &secret_key);

// 		let mut token_x_id = [0u8; 32];
// 		OsRng.fill_bytes(&mut token_x_id);
// 		let mut token_y_id = [0u8; 32];
// 		OsRng.fill_bytes(&mut token_y_id);
// 		let token_x_id = token_x_id.to_vec();
// 		let token_y_id = token_y_id.to_vec();
// 		let pk: [u8; 64] = public_key.serialize_uncompressed()[1..].try_into().expect("");
// 		let account = get_account_id_from_pk(public_key.serialize().as_slice());
// 		fund_account(account);

// 		// Create token_x
// 		assert_ok!(Assets::create_token(
// 			RuntimeOrigin::signed(1),
// 			pk,
// 			token_x_id.clone(),
// 			Some(Vec::<(u32, Vec<u8>)>::new()),
// 			None
// 		));

// 		// Create token_y
// 		assert_ok!(Assets::create_token(
// 			RuntimeOrigin::signed(1),
// 			pk,
// 			token_y_id.clone(),
// 			Some(Vec::<(u32, Vec<u8>)>::new()),
// 			None
// 		));

// 		let token_x_amount = 100000000000000u128;
// 		let token_y_amount = 1000000000000u128;

// 		let mut nonce = 0u128;
// 		// Mint and deposit token x
// 		mint(&secp, &token_x_id, &(secret_key, public_key), &pk, token_x_amount, nonce);
// 		nonce += 1;
// 		deposit(&secp, &token_x_id, &(secret_key, public_key), token_x_amount, nonce);
// 		assert_ok!(OmniSwap::deposit_comfirm(
// 			RuntimeOrigin::signed(1),
// 			pk,
// 			token_x_id.clone(),
// 			nonce
// 		));

// 		// Mint and deposit token y
// 		nonce += 1;
// 		mint(&secp, &token_y_id.to_vec(), &(secret_key, public_key), &pk, token_y_amount, nonce);
// 		nonce += 1;
// 		deposit(&secp, &token_y_id, &(secret_key, public_key), token_y_amount, nonce);
// 		assert_ok!(OmniSwap::deposit_comfirm(
// 			RuntimeOrigin::signed(1),
// 			pk,
// 			token_y_id.clone(),
// 			nonce
// 		));
// 		let deposit_x_amount = OmniSwap::balance(&pk, &token_x_id).unwrap_or(0);
// 		let deposit_y_amount = OmniSwap::balance(&pk, &token_y_id).unwrap_or(0);
// 		assert_eq!(deposit_x_amount, token_x_amount);
// 		assert_eq!(deposit_y_amount, token_y_amount);
// 		let trading_pair = vec![1];
// 		assert_ok!(
// 			OmniSwap::add_liquidity(
// 				RuntimeOrigin::signed(account),
// 				trading_pair.clone(),
// 				pk,
// 				deposit_x_amount,
// 				deposit_y_amount,
// 				deposit_x_amount,
// 				deposit_y_amount,
// 				token_x_id.clone(),
// 				token_y_id.clone()
// 			),
// 			()
// 		);
// 		assert_eq!(
// 			OmniSwap::trading_pairs(&trading_pair),
// 			Some((deposit_x_amount, deposit_y_amount))
// 		);
// 		let deposit_x_amount = OmniSwap::balance(&pk, &token_x_id).unwrap_or(0);
// 		let deposit_y_amount = OmniSwap::balance(&pk, &token_y_id).unwrap_or(0);
// 		assert_eq!(deposit_x_amount, 0);
// 		assert_eq!(deposit_y_amount, 0);
// 		assert_eq!(OmniSwap::total_liquidity(&trading_pair), Some(9999999999000));
// 		assert_eq!(OmniSwap::liquidity((trading_pair, pk)), Some(9999999999000));
// 	});
// }

// #[test]
// fn it_works_for_swap_x2y() {
// 	new_test_ext().execute_with(|| {
// 		let secp = Secp256k1::new();
// 		// Generate key pair
// 		// let (secret_key, public_key) = secp.generate_keypair(&mut OsRng);
// 		let secret_key = SecretKey::from_slice(&SECRET_KEY).unwrap();
// 		let public_key = PublicKey::from_secret_key(&secp, &secret_key);

// 		let mut token_x_id = [0u8; 32];
// 		OsRng.fill_bytes(&mut token_x_id);
// 		let mut token_y_id = [0u8; 32];
// 		OsRng.fill_bytes(&mut token_y_id);
// 		let token_x_id = token_x_id.to_vec();
// 		let token_y_id = token_y_id.to_vec();
// 		let pk: [u8; 64] = public_key.serialize_uncompressed()[1..].try_into().expect("");
// 		let account = get_account_id_from_pk(public_key.serialize().as_slice());
// 		fund_account(account);

// 		// Create token_x
// 		assert_ok!(Assets::create_token(
// 			RuntimeOrigin::signed(1),
// 			pk,
// 			token_x_id.clone(),
// 			Some(Vec::<(u32, Vec<u8>)>::new()),
// 			None
// 		));

// 		// Create token_y
// 		assert_ok!(Assets::create_token(
// 			RuntimeOrigin::signed(1),
// 			pk,
// 			token_y_id.clone(),
// 			Some(Vec::<(u32, Vec<u8>)>::new()),
// 			None
// 		));

// 		let swap_amount = 1000u128;
// 		let add_liquidity_amount = 1000000u128;
// 		let token_x_amount = swap_amount + add_liquidity_amount;
// 		let token_y_amount = 10000u128;

// 		let mut nonce = 0u128;
// 		// Mint and deposit token x
// 		mint(&secp, &token_x_id, &(secret_key, public_key), &pk, token_x_amount, nonce);
// 		nonce += 1;
// 		deposit(&secp, &token_x_id, &(secret_key, public_key), token_x_amount, nonce);
// 		assert_ok!(OmniSwap::deposit_comfirm(
// 			RuntimeOrigin::signed(1),
// 			pk,
// 			token_x_id.clone(),
// 			nonce
// 		));

// 		// Mint and deposit token y
// 		nonce += 1;
// 		mint(&secp, &token_y_id.to_vec(), &(secret_key, public_key), &pk, token_y_amount, nonce);
// 		nonce += 1;
// 		deposit(&secp, &token_y_id, &(secret_key, public_key), token_y_amount, nonce);
// 		assert_ok!(OmniSwap::deposit_comfirm(
// 			RuntimeOrigin::signed(1),
// 			pk,
// 			token_y_id.clone(),
// 			nonce
// 		));
// 		let deposit_x_amount = OmniSwap::balance(&pk, &token_x_id).unwrap_or(0);
// 		let deposit_y_amount = OmniSwap::balance(&pk, &token_y_id).unwrap_or(0);
// 		assert_eq!(deposit_x_amount, token_x_amount);
// 		assert_eq!(deposit_y_amount, token_y_amount);
		
// 		let trading_pair = vec![1];
// 		assert_ok!(
// 			OmniSwap::add_liquidity(
// 				RuntimeOrigin::signed(account),
// 				trading_pair.clone(),
// 				pk,
// 				add_liquidity_amount,
// 				deposit_y_amount,
// 				100,
// 				1,
// 				token_x_id.clone(),
// 				token_y_id.clone()
// 			),
// 			()
// 		);
// 		assert_eq!(OmniSwap::balance(&pk, &token_x_id).unwrap_or(0), swap_amount);
// 		assert_eq!(OmniSwap::balance(&pk, &token_y_id).unwrap_or(0), 0);
// 		assert_ok!(
// 			OmniSwap::swap_x2y(
// 				RuntimeOrigin::signed(account),
// 				trading_pair.clone(),
// 				pk,
// 				swap_amount,
// 				1
// 			),
// 			()
// 		);
// 		assert_eq!(OmniSwap::trading_pairs(&trading_pair), Some((1001000, 9991)));
// 		assert_eq!(OmniSwap::balance(&pk, &token_x_id).unwrap_or(0), 0);
// 		assert_eq!(OmniSwap::balance(&pk, &token_y_id).unwrap_or(0), 9);
// 	});
// }

#[test]
fn it_works_for_swap_y2x() {
	new_test_ext().execute_with(|| {
		let secp = Secp256k1::new();
		// Generate key pair
		// let (secret_key, public_key) = secp.generate_keypair(&mut OsRng);
		let secret_key = SecretKey::from_slice(&SECRET_KEY).unwrap();
		let public_key = PublicKey::from_secret_key(&secp, &secret_key);

		let mut token_x_id = [0u8; 32];
		OsRng.fill_bytes(&mut token_x_id);
		let mut token_y_id = [0u8; 32];
		OsRng.fill_bytes(&mut token_y_id);
		let token_x_id = token_x_id.to_vec();
		let token_y_id = token_y_id.to_vec();
		let pk: [u8; 64] = public_key.serialize_uncompressed()[1..].try_into().expect("");
		let account = get_account_id_from_pk(public_key.serialize().as_slice());
		fund_account(account);

		// Create token_x
		assert_ok!(Assets::create_token(
			RuntimeOrigin::signed(1),
			pk,
			token_x_id.clone(),
			Some(Vec::<(u32, Vec<u8>)>::new()),
			None
		));

		// Create token_y
		assert_ok!(Assets::create_token(
			RuntimeOrigin::signed(1),
			pk,
			token_y_id.clone(),
			Some(Vec::<(u32, Vec<u8>)>::new()),
			None
		));

		let swap_amount = 10u128;
		let add_liquidity_amount = 10000u128;
		let token_x_amount = 1000000u128;
		let token_y_amount = swap_amount + add_liquidity_amount;

		let mut nonce = 0u128;
		// Mint and deposit token x
		mint(&secp, &token_x_id, &(secret_key, public_key), &pk, token_x_amount, nonce);
		nonce += 1;
		deposit(&secp, &token_x_id, &(secret_key, public_key), token_x_amount, nonce);
		assert_ok!(OmniSwap::deposit_comfirm(
			RuntimeOrigin::signed(1),
			pk,
			token_x_id.clone(),
			nonce
		));

		// Mint and deposit token y
		nonce += 1;
		mint(&secp, &token_y_id.to_vec(), &(secret_key, public_key), &pk, token_y_amount, nonce);
		nonce += 1;
		deposit(&secp, &token_y_id, &(secret_key, public_key), token_y_amount, nonce);
		assert_ok!(OmniSwap::deposit_comfirm(
			RuntimeOrigin::signed(1),
			pk,
			token_y_id.clone(),
			nonce
		));
		let deposit_x_amount = OmniSwap::balance(&pk, &token_x_id).unwrap_or(0);
		let deposit_y_amount = OmniSwap::balance(&pk, &token_y_id).unwrap_or(0);
		assert_eq!(deposit_x_amount, token_x_amount);
		assert_eq!(deposit_y_amount, token_y_amount);

		let trading_pair = vec![1];
		assert_ok!(
			OmniSwap::add_liquidity(
				RuntimeOrigin::signed(account),
				trading_pair.clone(),
				pk,
				deposit_x_amount,
				add_liquidity_amount,
				100,
				1,
				token_x_id.clone(),
				token_y_id.clone()
			),
			()
		);
		assert_eq!(OmniSwap::balance(&pk, &token_x_id).unwrap_or(0), 0);
		assert_eq!(OmniSwap::balance(&pk, &token_y_id).unwrap_or(0), swap_amount);

		assert_ok!(
			OmniSwap::swap_y2x(
				RuntimeOrigin::signed(account),
				trading_pair.clone(),
				pk,
				swap_amount,
				1
			),
			()
		);
		assert_eq!(OmniSwap::trading_pairs(&trading_pair), Some((999001, 10010)));
		assert_eq!(OmniSwap::balance(&pk, &token_x_id).unwrap_or(0), 999);
		assert_eq!(OmniSwap::balance(&pk, &token_y_id).unwrap_or(0), 0);
	});
}
