use super::traits::*;
use super::*;
use crate::OmniverseTransactionData;
use frame_support::traits::{Get, UnixTime};
use sp_core::Hasher;
use sp_io::crypto;
use sp_runtime::traits::Keccak256;
use sp_std::vec::Vec;

pub fn get_transaction_hash(data: &OmniverseTransactionData) -> [u8; 32] {
	let mut raw = Vec::<u8>::new();
	raw.extend_from_slice(&mut u128::to_be_bytes(data.nonce).as_slice());
	raw.extend_from_slice(&mut u32::to_be_bytes(data.chain_id).as_slice());
	raw.extend_from_slice(&mut data.from.clone());
	raw.append(&mut data.initiator_address.clone().as_mut());

	let mut bytes_data = Vec::<u8>::new();
	// let op_data = TokenOpcode::decode(&mut data.op_data.as_slice()).unwrap();
	bytes_data.extend_from_slice(&mut u8::to_be_bytes(data.op_type).as_slice());

	// if data.op_type == TRANSFER {
	// 	// let transfer_data = TransferTokenOp::decode(&mut data.op_data.as_slice()).unwrap();
	// 	bytes_data.extend(data.op_data.clone());
	// 	bytes_data.extend_from_slice(&mut u128::to_be_bytes(data.amount).as_slice());
	// } else if data.op_type == MINT {
	// 	let mint_data = MintTokenOp::decode(&mut data.op_data.as_slice()).unwrap();
	// 	bytes_data.extend_from_slice(&mut mint_data.to.clone());
	// 	bytes_data.extend_from_slice(&mut u128::to_be_bytes(mint_data.amount).as_slice());
	// }
	bytes_data.extend(data.op_data.clone());
	bytes_data.extend_from_slice(&mut u128::to_be_bytes(data.amount).as_slice());
	raw.append(&mut bytes_data.as_mut());

	let h = Keccak256::hash(raw.as_slice());

	h.0
}

impl<T: Config> OmniverseAccounts for Pallet<T> {
	fn verify_transaction(
		token_id: &Vec<u8>,
		data: &OmniverseTransactionData,
	) -> Result<VerifyResult, VerifyError> {
		let nonce = TransactionCount::<T>::get(&data.from, token_id);

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
			TransactionCount::<T>::insert(&data.from, token_id, nonce + 1);
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

	fn get_transaction_count(pk: [u8; 64], token_id: Vec<u8>) -> u128 {
		Self::transaction_count(pk, token_id)
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

	fn get_chain_id() -> u32 {
		T::ChainId::get()
	}

	fn get_transaction_data(pk: [u8; 64], nonce: u128) -> Option<OmniverseTx> {
		TransactionRecorder::<T>::get(pk, nonce)
	}

	fn get_cooling_down_time() -> u64 {
		10
	}
}
