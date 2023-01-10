use super::*;
use super::traits::*;
use crate::{MintTokenOp, OmniverseTokenProtocol, TokenOpcode, TransferTokenOp, MINT, TRANSFER};
use codec::Decode;
use sp_core::Hasher;
use frame_support::traits::{Get, UnixTime};
use sp_runtime::{traits::Keccak256};
use sp_std::vec::Vec;
use sp_io::crypto;

pub fn get_transaction_hash(data: &OmniverseTokenProtocol) -> [u8; 32] {
	let mut raw = Vec::<u8>::new();
	raw.extend_from_slice(&mut u128::to_be_bytes(data.nonce).as_slice());
	raw.extend_from_slice(&mut u8::to_be_bytes(data.chain_id).as_slice());
	raw.extend_from_slice(&mut data.from.clone());
	raw.append(&mut data.to.clone().as_mut());

	let mut bytes_data = Vec::<u8>::new();
	let op_data = TokenOpcode::decode(&mut data.data.as_slice()).unwrap();
	bytes_data.extend_from_slice(&mut u8::to_be_bytes(op_data.op).as_slice());

	if op_data.op == TRANSFER {
		let transfer_data = TransferTokenOp::decode(&mut op_data.data.as_slice()).unwrap();
		bytes_data.extend_from_slice(&mut transfer_data.to.clone());
		bytes_data.extend_from_slice(&mut u128::to_be_bytes(transfer_data.amount).as_slice());
	} else if op_data.op == MINT {
		let mint_data = MintTokenOp::decode(&mut op_data.data.as_slice()).unwrap();
		bytes_data.extend_from_slice(&mut mint_data.to.clone());
		bytes_data.extend_from_slice(&mut u128::to_be_bytes(mint_data.amount).as_slice());
	}
	raw.append(&mut bytes_data.as_mut());

	let h = Keccak256::hash(raw.as_slice());

	h.0
}

impl<T: Config> OmniverseAccounts for Pallet<T> {
	fn verify_transaction(data: &OmniverseTokenProtocol) -> Result<VerifyResult, VerifyError> {
		let nonce = TransactionCount::<T>::get(&data.from);

		let tx_hash_bytes = super::functions::get_transaction_hash(&data);

		let recoverd_pk = crypto::secp256k1_ecdsa_recover(&data.signature, &tx_hash_bytes)
			.map_err(|_| VerifyError::SignatureError)?;

		if recoverd_pk != data.from {
			return Err(VerifyError::SignerNotCaller);
		}

		// Check nonce
		if nonce == data.nonce {
			// Add to transaction recorder
			let omni_tx = OmniverseTx::new(data.clone(), T::Timestamp::now().as_secs());
			TransactionRecorder::<T>::insert(&data.from, &nonce, omni_tx);
			TransactionCount::<T>::insert(&data.from, nonce + 1);
			if data.chain_id == T::ChainId::get() {
				Self::deposit_event(Event::TransactionSent(data.from, nonce));
			}
			Ok(VerifyResult::Success)
		} else if nonce > data.nonce {
			// Check conflicts
			let his_tx = TransactionRecorder::<T>::get(&data.from, &data.nonce).unwrap();
			let his_tx_hash = super::functions::get_transaction_hash(&his_tx.tx_data);
			if his_tx_hash != tx_hash_bytes {
				let omni_tx = OmniverseTx::new(data.clone(), T::Timestamp::now().as_secs());
				let evil_tx = EvilTxData::new(omni_tx, nonce);
				let mut er =
					EvilRecorder::<T>::get(&data.from).unwrap_or(Vec::<EvilTxData>::default());
				er.push(evil_tx);
				EvilRecorder::<T>::insert(&data.from, er);
				Ok(VerifyResult::Malicious)
			} else {
				Ok(VerifyResult::Duplicated)
			}
		} else {
			Err(VerifyError::NonceError)
		}
	}

	fn get_transaction_count(pk: [u8; 64]) -> u128 {
		Self::transaction_count(pk)
	}

	fn is_malicious(pk: [u8; 64]) -> bool {
		let record = Self::evil_recorder(pk);
		if let Some(r) = record {
			if r.len() > 0 {
				return true;
			}
		}

		false
	}

	fn get_chain_id() -> u8 {
		T::ChainId::get()
	}
	
	fn get_transaction_data(pk: [u8; 64], nonce: u128) -> Option<OmniverseTx> {
		TransactionRecorder::<T>::get(pk, nonce)
	}

	fn get_cooling_down_time() -> u64 {
		10
	}
}