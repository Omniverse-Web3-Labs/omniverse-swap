use crate::{DispatchError, FactoryResult};
use pallet_omniverse_protocol::types::OmniverseTransactionData;
use sp_std::vec::Vec;

pub trait OmniverseTokenFactoryHandler {
	fn send_transaction_external(
		token_id: Vec<u8>,
		data: &OmniverseTransactionData,
	) -> Result<FactoryResult, DispatchError>;
}
