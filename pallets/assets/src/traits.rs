use pallet_omniverse_protocol::types::OmniverseTokenProtocol;
use sp_std::vec::Vec;
use crate::{DispatchError, FactoryResult};

pub trait OmniverseTokenFactoryHandler {
    fn send_transaction_external(token_id: Vec<u8>, data: &OmniverseTokenProtocol) -> Result<FactoryResult, DispatchError>;
}