#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::unnecessary_mut_passed)]

use omniverse_protocol_traits::{OmniverseAccounts, OmniverseTokenProtocol};
use sp_std::vec::Vec;
use codec::{Encode, Decode};

#[derive(Decode, Encode)]
pub struct TokenOpcode {
    pub op: u8,
    pub data: Vec<u8>
}

impl TokenOpcode {
    pub fn new(op: u8, data: Vec<u8>) -> Self {
        Self {
            op,
            data,
        }
    }
}

#[derive(Decode, Encode)]
pub struct MintTokenOp {
    pub to: [u8; 64],
    pub amount: u128
}

impl MintTokenOp {
    pub fn new(to: [u8; 64], amount: u128) -> Self {
        Self {
            to,
            amount,
        }
    }
}

#[derive(Decode, Encode)]
pub struct TransferTokenOp {
    pub to: [u8; 64],
    pub amount: u128
}

impl TransferTokenOp {
    pub fn new(to: [u8; 64], amount: u128) -> Self {
        Self {
            to,
            amount,
        }
    }
}

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