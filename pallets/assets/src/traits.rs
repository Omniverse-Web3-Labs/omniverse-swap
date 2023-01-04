#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::unnecessary_mut_passed)]

use pallet_omniverse_protocol::types::OmniverseTokenProtocol;
use sp_std::vec::Vec;
use crate::{DispatchError, FactoryResult};

pub trait OmniverseTokenFactoryHandler {
    fn send_transaction_external(token_id: Vec<u8>, data: &OmniverseTokenProtocol) -> Result<FactoryResult, DispatchError>;
}