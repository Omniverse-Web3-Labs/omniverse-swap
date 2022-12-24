#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::unnecessary_mut_passed)]

use omniverse_protocol_traits::{OmniverseAccounts, OmniverseTokenProtocol};
use sp_std::vec::Vec;

#[derive(Debug, PartialEq)]
pub enum FactoryError {
    TokenNotExist,
    WrongDestination,
    UserIsMalicious,
    SignatureError,
    BalanceOverflow,
    SignerNotOwner,
}

pub trait OmniverseTokenFactoryHandler {
    fn send_transaction_external(token_id: Vec<u8>, data: &OmniverseTokenProtocol) -> Result<(), FactoryError>;
}