use crate::{OmniverseTokenProtocol, VerifyError, VerifyResult, OmniverseTx};

pub trait OmniverseAccounts {
	fn verify_transaction(data: &OmniverseTokenProtocol) -> Result<VerifyResult, VerifyError>;
	fn get_transaction_count(pk: [u8; 64]) -> u128;
	fn is_malicious(pk: [u8; 64]) -> bool;
	fn get_chain_id() -> u8;
	fn get_transaction_data(pk: [u8; 64], nonce: u128) -> Option<OmniverseTx>;
	fn get_cooling_down_time() -> u64;
}
