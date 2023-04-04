use crate::{
	mock::*, traits::OmniverseAccounts, Fungible, OmniverseTransactionData, VerifyError,
	VerifyResult, MINT, TRANSFER,
};
use codec::Encode;
use frame_support::assert_err;
use secp256k1::rand::rngs::OsRng;
use secp256k1::{ecdsa::RecoverableSignature, Message, PublicKey, Secp256k1, SecretKey};
use sp_core::Hasher;
use sp_runtime::traits::Keccak256;

const CHAIN_ID: u32 = 1;
const INITIATOR_ADDRESS: Vec<u8> = Vec::<u8>::new();
const PALLET_NAME: Vec<u8> = Vec::<u8>::new();

fn get_sig_slice(sig: &RecoverableSignature) -> [u8; 65] {
	let (recovery_id, sig_slice) = sig.serialize_compact();
	let mut sig_recovery: [u8; 65] = [0; 65];
	sig_recovery[0..64].copy_from_slice(&sig_slice);
	sig_recovery[64] = recovery_id.to_i32() as u8;
	sig_recovery
}

fn encode_transaction(
	secp: &Secp256k1<secp256k1::All>,
	from: (SecretKey, PublicKey),
	nonce: u128,
	amount: u128,
	with_ethereum: bool,
) -> OmniverseTransactionData {
	let pk: [u8; 64] = from.1.serialize_uncompressed()[1..].try_into().expect("");
	let payload = Fungible::new(TRANSFER, pk.into(), amount).encode();
	// let op_data = TokenOpcode::new(TRANSFER, transfer_data).encode();
	encode_transaction_with_data(secp, from, nonce, payload, with_ethereum)
}

fn encode_transaction_with_data(
	secp: &Secp256k1<secp256k1::All>,
	from: (SecretKey, PublicKey),
	nonce: u128,
	payload: Vec<u8>,
	with_ethereum: bool,
) -> OmniverseTransactionData {
	let pk: [u8; 64] = from.1.serialize_uncompressed()[1..].try_into().expect("");
	let mut tx_data =
		OmniverseTransactionData::new(nonce, CHAIN_ID, INITIATOR_ADDRESS, pk, payload);
	let h = tx_data.get_raw_hash(with_ethereum);
	let message = Message::from_slice(h.as_slice())
		.expect("messages must be 32 bytes and are expected to be hashes");
	let sig: RecoverableSignature = secp.sign_ecdsa_recoverable(&message, &from.0);
	let sig_recovery = get_sig_slice(&sig);
	tx_data.set_signature(sig_recovery);
	tx_data
}

#[test]
fn it_fails_for_signature_error() {
	new_test_ext().execute_with(|| {
		let secp = Secp256k1::new();
		// Generate key pair
		let (secret_key, public_key) = secp.generate_keypair(&mut OsRng);

		// Get nonce
		let pk: [u8; 64] = public_key.serialize_uncompressed()[1..].try_into().expect("");
		let nonce = OmniverseProtocol::get_transaction_count(pk, PALLET_NAME, Vec::new());
		let amount: u128 = 1;

		// Encode transaction
		let mut data = encode_transaction(&secp, (secret_key, public_key), nonce, amount, false);

		// Set a wrong signature
		data.set_signature([0; 65]);

		assert_err!(
			OmniverseProtocol::verify_transaction(&PALLET_NAME, &Vec::new(), &data, false),
			VerifyError::SignatureError
		);
	});
}

#[test]
fn it_fails_for_signer_not_caller_error() {
	new_test_ext().execute_with(|| {
		let secp = Secp256k1::new();
		// Generate key pair
		let (_, public_key) = secp.generate_keypair(&mut OsRng);

		// Get nonce
		let pk: [u8; 64] = public_key.serialize_uncompressed()[1..].try_into().expect("");
		let nonce = OmniverseProtocol::get_transaction_count(pk, PALLET_NAME, Vec::new());
		let amount = 1;
		// Encode transaction
		let (new_secret_key, _) = secp.generate_keypair(&mut OsRng);
		let data = encode_transaction(&secp, (new_secret_key, public_key), nonce, amount, false);

		assert_err!(
			OmniverseProtocol::verify_transaction(&PALLET_NAME, &Vec::new(), &data, false),
			VerifyError::SignerNotCaller
		);
	});
}

#[test]
fn it_fails_for_nonce_error() {
	new_test_ext().execute_with(|| {
		let secp = Secp256k1::new();
		// Generate key pair
		let (secret_key, public_key) = secp.generate_keypair(&mut OsRng);

		// Get nonce
		let pk: [u8; 64] = public_key.serialize_uncompressed()[1..].try_into().expect("");
		let nonce = OmniverseProtocol::get_transaction_count(pk, PALLET_NAME, Vec::new()) + 1;
		let amount = 1;
		// Encode transaction
		let data = encode_transaction(&secp, (secret_key, public_key), nonce, amount, false);

		assert_err!(
			OmniverseProtocol::verify_transaction(&PALLET_NAME, &Vec::new(), &data, false),
			VerifyError::NonceError
		);
	});
}

#[test]
fn it_works_for_verify_transaction() {
	new_test_ext().execute_with(|| {
		let secp = Secp256k1::new();
		// Generate key pair
		let (secret_key, public_key) = secp.generate_keypair(&mut OsRng);

		// Get nonce
		let pk: [u8; 64] = public_key.serialize_uncompressed()[1..].try_into().expect("");
		let nonce = OmniverseProtocol::get_transaction_count(pk, PALLET_NAME, Vec::new());
		let amount = 1;

		// Encode transaction
		let data = encode_transaction(&secp, (secret_key, public_key), nonce, amount, false);

		let ret = OmniverseProtocol::verify_transaction(&PALLET_NAME, &Vec::new(), &data, false);
		assert!(ret.is_ok());
		assert_eq!(ret.unwrap(), VerifyResult::Success);
	});
}

#[test]
fn it_works_for_malicious_transaction() {
	new_test_ext().execute_with(|| {
		let secp = Secp256k1::new();
		// Generate key pair
		let (secret_key, public_key) = secp.generate_keypair(&mut OsRng);

		// Get nonce
		let pk: [u8; 64] = public_key.serialize_uncompressed()[1..].try_into().expect("");
		let nonce = OmniverseProtocol::get_transaction_count(pk, PALLET_NAME, Vec::new());
		let amount = 1;

		// Encode transaction
		let data = encode_transaction(&secp, (secret_key, public_key), nonce, amount, false);

		let ret = OmniverseProtocol::verify_transaction(&PALLET_NAME, &Vec::new(), &data, false);
		assert!(ret.is_ok());
		assert_eq!(ret.unwrap(), VerifyResult::Success);
		// Encode a malicious transaction
		// let op_data = TransferTokenOp::new(pk, amount).encode();
		let payload = Fungible::new(MINT, pk.into(), amount).encode();
		// let op_data = TokenOpcode::new(TRANSFER, transfer_data).encode();
		let data_new =
			encode_transaction_with_data(&secp, (secret_key, public_key), nonce, payload, false);

		let ret =
			OmniverseProtocol::verify_transaction(&PALLET_NAME, &Vec::new(), &data_new, false);
		assert!(ret.is_ok());
		assert_eq!(ret.unwrap(), VerifyResult::Malicious);
	});
}

#[test]
fn it_works_for_duplicated_transaction() {
	new_test_ext().execute_with(|| {
		let secp = Secp256k1::new();
		// Generate key pair
		let (secret_key, public_key) = secp.generate_keypair(&mut OsRng);

		// Get nonce
		let pk: [u8; 64] = public_key.serialize_uncompressed()[1..].try_into().expect("");
		let nonce = OmniverseProtocol::get_transaction_count(pk, PALLET_NAME, Vec::new());
		let amount = 1;

		// Encode transaction
		let data = encode_transaction(&secp, (secret_key, public_key), nonce, amount, false);

		let ret = OmniverseProtocol::verify_transaction(&PALLET_NAME, &Vec::new(), &data, false);
		assert!(ret.is_ok());
		assert_eq!(ret.unwrap(), VerifyResult::Success);

		let ret = OmniverseProtocol::verify_transaction(&PALLET_NAME, &Vec::new(), &data, false);
		assert!(ret.is_ok());
		assert_eq!(ret.unwrap(), VerifyResult::Duplicated);
	});
}

#[test]
fn it_works_for_ethereum_signature() {
	new_test_ext().execute_with(|| {
		let secp = Secp256k1::new();
		// pub fn seeded_randomize(&mut self, seed: &[u8; 32]) {
		let secret_key = SecretKey::from_slice(&[
			142, 190, 93, 31, 248, 244, 136, 11, 92, 255, 107, 224, 114, 68, 27, 173, 40, 169, 129,
			200, 15, 254, 183, 85, 187, 160, 40, 250, 193, 56, 118, 245,
		])
		.expect("32 bytes, within curve order");
		// Generate key pair
		// let public_key = secret_key.public_key(&secp);
		let mut raw = String::from("hello").into_bytes();
		let etherum_prefix = String::from("\x19Ethereum Signed Message:\n");
		let prefix = etherum_prefix + &raw.len().to_string();
		let mut prefix_vec = prefix.as_bytes().to_vec();
		// raw.prepend(prefix.as_bytes());
		prefix_vec.extend(raw);
		raw = prefix_vec;
		let h = Keccak256::hash(raw.as_slice());
		let message = Message::from_slice(h.0.as_slice())
			.expect("messages must be 32 bytes and are expected to be hashes");
		let sig: RecoverableSignature = secp.sign_ecdsa_recoverable(&message, &secret_key);
		let signature = get_sig_slice(&sig);
		let expect: [u8; 65] = [
			165, 24, 166, 79, 196, 100, 34, 122, 191, 167, 30, 91, 208, 70, 76, 217, 156, 253, 210,
			165, 97, 40, 107, 117, 83, 47, 89, 93, 3, 190, 45, 128, 58, 99, 59, 116, 14, 64, 232,
			108, 191, 50, 201, 27, 153, 90, 182, 109, 80, 18, 183, 15, 34, 92, 178, 234, 37, 57,
			220, 198, 183, 204, 103, 124, 0,
		];
		assert_eq!(signature, expect);
	});
}
