use crate::{OmniverseTransactionData, OmniverseTx, VerifyError, VerifyResult};
use sp_std::vec::Vec;

pub trait OmniverseAccounts {
	fn verify_transaction(
		token_id: &Vec<u8>,
		data: &OmniverseTransactionData,
	) -> Result<VerifyResult, VerifyError>;
	fn get_transaction_count(pk: [u8; 64], token_id: Vec<u8>) -> u128;
	fn is_malicious(pk: [u8; 64]) -> bool;
	fn get_chain_id() -> u32;
	fn get_transaction_data(pk: [u8; 64], nonce: u128) -> Option<OmniverseTx>;
	fn get_cooling_down_time() -> u64;
}
