use crate::functions;
use codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_std::vec::Vec;

pub const TRANSFER: u8 = 0_u8;
pub const MINT: u8 = 1_u8;
pub const BURN: u8 = 2_u8;

// #[derive(Decode, Encode, Debug)]
// pub struct TokenOpcode {
// 	pub op: u8,
// 	pub data: Vec<u8>,
// }

// impl TokenOpcode {
// 	pub fn new(op: u8, data: Vec<u8>) -> Self {
// 		Self { op, data }
// 	}
// }

#[derive(Decode, Encode, Debug)]
pub struct Fungible {
	pub op: u8,
	pub ex_data: Vec<u8>,
	pub amount: u128,
}

impl Fungible {
	pub fn new(op: u8, ex_data: Vec<u8>, amount: u128) -> Self {
		Self { op, ex_data, amount }
	}
}

#[derive(Decode, Encode, Debug)]
pub struct Assets {
	pub op: u8,
	pub ex_data: Vec<u8>,
	pub quantity: u128,
}

impl Assets {
	pub fn new(op: u8, ex_data: Vec<u8>, quantity: u128) -> Self {
		Self { op, ex_data, quantity }
	}
}

#[derive(Decode, Encode, Debug)]
pub struct NonFungible {
	pub op: u8,
	pub ex_data: Vec<u8>,
	pub token_id: u128,
}

#[derive(Decode, Encode, Debug)]
pub struct MintTokenOp {
	pub to: [u8; 64],
	pub amount: u128,
}

impl MintTokenOp {
	pub fn new(to: [u8; 64], amount: u128) -> Self {
		Self { to, amount }
	}
}

#[derive(Decode, Encode, Debug)]
pub struct TransferTokenOp {
	pub to: [u8; 64],
	pub amount: u128,
}

impl TransferTokenOp {
	pub fn new(to: [u8; 64], amount: u128) -> Self {
		Self { to, amount }
	}
}

#[derive(Clone, PartialEq, Eq, Debug, Encode, Decode, TypeInfo)]
pub enum VerifyResult {
	Success,
	Malicious,
	Duplicated,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum VerifyError {
	SignatureError,
	NonceError,
	SignerNotCaller,
}

#[derive(Clone, PartialEq, Eq, Debug, Encode, Decode, TypeInfo)]
pub struct OmniverseTransactionData {
	pub nonce: u128,
	pub chain_id: u32,
	pub initiator_address: Vec<u8>,
	pub from: [u8; 64],
	pub payload: Vec<u8>,
	pub signature: [u8; 65],
}

impl OmniverseTransactionData {
	pub fn new(
		nonce: u128,
		chain_id: u32,
		initiator_address: Vec<u8>,
		from: [u8; 64],
		payload: Vec<u8>,
	) -> Self {
		Self { nonce, chain_id, initiator_address, from, payload, signature: [0; 65] }
	}

	pub fn get_raw_hash(&self, with_ethereum: bool) -> [u8; 32] {
		functions::get_transaction_hash(self, with_ethereum)
	}

	pub fn set_signature(&mut self, signature: [u8; 65]) {
		self.signature = signature;
	}
}

#[derive(Clone, PartialEq, Eq, Debug, Encode, Decode, TypeInfo)]
pub struct OmniverseTx {
	pub tx_data: OmniverseTransactionData,
	pub timestamp: u64,
}

impl OmniverseTx {
	pub fn new(data: OmniverseTransactionData, timestamp: u64) -> Self {
		Self { tx_data: data, timestamp }
	}
}

#[derive(Clone, PartialEq, Eq, Debug, Encode, Decode, TypeInfo)]
pub struct EvilTxData {
	pub tx_omni: OmniverseTx,
	pub his_nonce: u128,
}

impl EvilTxData {
	pub fn new(data: OmniverseTx, nonce: u128) -> Self {
		Self { tx_omni: data, his_nonce: nonce }
	}
}
