use crate::{
	mock::*, traits::OmniverseAccounts, Fungible, OmniverseTransactionData, VerifyError,
	VerifyResult, MINT, TRANSFER,
};
use codec::Encode;
use frame_support::assert_err;
use secp256k1::rand::rngs::OsRng;
use secp256k1::{ecdsa::RecoverableSignature, Message, PublicKey, Secp256k1, SecretKey};

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
) -> OmniverseTransactionData {
	let pk: [u8; 64] = from.1.serialize_uncompressed()[1..].try_into().expect("");
	let payload = Fungible::new(TRANSFER, pk.into(), amount).encode();
	// let op_data = TokenOpcode::new(TRANSFER, transfer_data).encode();
	encode_transaction_with_data(secp, from, nonce, payload)
}

fn encode_transaction_with_data(
	secp: &Secp256k1<secp256k1::All>,
	from: (SecretKey, PublicKey),
	nonce: u128,
	payload: Vec<u8>,
) -> OmniverseTransactionData {
	let pk: [u8; 64] = from.1.serialize_uncompressed()[1..].try_into().expect("");
	let mut tx_data =
		OmniverseTransactionData::new(nonce, CHAIN_ID, INITIATOR_ADDRESS, pk, payload);
	let h = tx_data.get_raw_hash();
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
		let mut data = encode_transaction(&secp, (secret_key, public_key), nonce, amount);

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
		let data = encode_transaction(&secp, (new_secret_key, public_key), nonce, amount);

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
		let data = encode_transaction(&secp, (secret_key, public_key), nonce, amount);

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
		let data = encode_transaction(&secp, (secret_key, public_key), nonce, amount);

		let ret = OmniverseProtocol::verify_transaction(&PALLET_NAME , &Vec::new(), &data, false);
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
		let data = encode_transaction(&secp, (secret_key, public_key), nonce, amount);

		let ret = OmniverseProtocol::verify_transaction(&PALLET_NAME, &Vec::new(), &data, false);
		assert!(ret.is_ok());
		assert_eq!(ret.unwrap(), VerifyResult::Success);
		// Encode a malicious transaction
		// let op_data = TransferTokenOp::new(pk, amount).encode();
		let payload = Fungible::new(MINT, pk.into(), amount).encode();
		// let op_data = TokenOpcode::new(TRANSFER, transfer_data).encode();
		let data_new =
			encode_transaction_with_data(&secp, (secret_key, public_key), nonce, payload);

		let ret = OmniverseProtocol::verify_transaction(&PALLET_NAME, &Vec::new(), &data_new, false);
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
		let data = encode_transaction(&secp, (secret_key, public_key), nonce, amount);

		let ret = OmniverseProtocol::verify_transaction(&PALLET_NAME, &Vec::new(), &data, false);
		assert!(ret.is_ok());
		assert_eq!(ret.unwrap(), VerifyResult::Success);

		let ret = OmniverseProtocol::verify_transaction(&PALLET_NAME, &Vec::new(), &data, false);
		assert!(ret.is_ok());
		assert_eq!(ret.unwrap(), VerifyResult::Duplicated);
	});
}
