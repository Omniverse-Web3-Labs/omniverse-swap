use crate::{OmniverseTransactionData, OmniverseTx, VerifyError, VerifyResult};
use sp_std::vec::Vec;

pub trait OmniverseAccounts {
	fn verify_transaction(
		pallet_name: &[u8],
		token_id: &[u8],
		data: &OmniverseTransactionData,
		with_ethereum: bool,
	) -> Result<VerifyResult, VerifyError>;
	fn get_transaction_count(pk: [u8; 64], pallet_name: Vec<u8>, token_id: Vec<u8>) -> u128;
	fn is_malicious(pk: [u8; 64]) -> bool;
	fn get_chain_id() -> u32;
	fn get_transaction_data(
		pk: [u8; 64],
		pallet_name: Vec<u8>,
		token_id: Vec<u8>,
		nonce: u128,
	) -> Option<OmniverseTx>;
	fn execute(
		pk: [u8; 64],
		pallet_name: Vec<u8>,
		token_id: Vec<u8>,
		nonce: u128,
	);
}
