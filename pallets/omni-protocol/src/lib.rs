#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {    
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use sp_std::vec::Vec;
	use codec::{Encode, Decode};
	use sp_core::{Hasher};
	use sp_io::crypto;
	use sp_runtime::{
		traits::{
			Keccak256
		}
	};

    /// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

    #[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

    // The pallet's runtime storage items.
	// https://docs.substrate.io/v3/runtime/storage
	#[pallet::storage]
	#[pallet::getter(fn transaction_recorder)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/v3/runtime/storage#declaring-storage-items
	pub type TransactionRecorder<T:Config> = StorageMap<_, Blake2_128Concat, [u8; 64], Vec<OmniverseTx>>;

	#[pallet::storage]
	#[pallet::getter(fn evil_recorder)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/v3/runtime/storage#declaring-storage-items
	pub type EvilRecorder<T:Config> = StorageMap<_, Blake2_128Concat, [u8; 64], Vec<EvilTxData>>;

    // Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
	}

    // Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
	}

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
	}

	const CHAIN_ID: u8 = 0_u8;
	
	#[derive(Clone, PartialEq, Eq, Debug, Encode, Decode, TypeInfo)]
	pub struct OmniverseTokenProtocol {
		nonce: u128,
		chain_id: u8,
		from: [u8; 64],
		to: Vec<u8>,
		data: Vec<u8>,
		signature: [u8; 65]
	}

	impl OmniverseTokenProtocol {
		pub fn new(nonce: u128, chain_id: u8, from: [u8; 64], to: Vec<u8>, data: Vec<u8>) -> Self {
			Self {
				nonce,
				chain_id,
				from,
				to,
				data,
				signature: [0; 65]
			}
		}

		pub fn get_raw_hash(&self) -> [u8; 32] {
			get_transaction_hash(self)
		}

		pub fn set_signature(&mut self, signature: [u8; 65]) {
			self.signature = signature;
		}
	}
	
	#[derive(Clone, PartialEq, Eq, Debug, Encode, Decode, TypeInfo)]
	pub struct OmniverseTx {
		tx_data: OmniverseTokenProtocol,
		timestamp: u128
	}
	
	impl OmniverseTx {
		fn new(data: OmniverseTokenProtocol) -> Self {
			Self {
				tx_data: data,
				timestamp: 0
			}
		}
	}
	
	#[derive(Clone, PartialEq, Eq, Debug, Encode, Decode, TypeInfo)]
	pub struct EvilTxData {
		tx_omni: OmniverseTx,
		his_nonce: u128
	}
	
	impl EvilTxData {
		fn new (data: OmniverseTx, nonce: u128) -> Self {
			Self {
				tx_omni: data,
				his_nonce: nonce
			}
		}
	}
	
	#[derive(Clone, PartialEq, Eq, Debug, Encode, Decode, TypeInfo)]
	pub enum VerifyResult {
		Success,
		Malicious,
		Duplicated
	}
	
	#[derive(Clone, PartialEq, Eq, Debug)]
	pub enum VerifyError {
		SignatureError,
		NonceError,
		SignerNotCaller,
	}

	pub trait OmniverseAccounts {
		fn verify_transaction(data: &OmniverseTokenProtocol) -> Result<VerifyResult, VerifyError>;
		fn get_transaction_count(pk: [u8; 64]) -> u128;
		fn is_malicious(pk: [u8; 64]) -> bool;
		fn get_chain_id() -> u8;
	}

	fn get_transaction_hash(data: &OmniverseTokenProtocol) -> [u8; 32] {
		let mut raw = Vec::<u8>::new();
		raw.extend_from_slice(&mut u128::to_be_bytes(data.nonce).as_slice());
		raw.extend_from_slice(&mut u8::to_be_bytes(data.chain_id).as_slice());
		raw.extend_from_slice(&mut data.from.clone());
		raw.append(&mut data.to.clone().as_mut());
		raw.append(&mut data.data.clone());
	
		let h = Keccak256::hash(raw.as_slice());
		
		h.0
	}

	impl<T: Config> OmniverseAccounts for Pallet<T> {
		fn verify_transaction(data: &OmniverseTokenProtocol) -> Result<VerifyResult, VerifyError> {
			let mut tr = TransactionRecorder::<T>::get(&data.from).unwrap_or(Vec::<OmniverseTx>::default());
			let nonce = tr.len() as u128;
	
			let tx_hash_bytes = get_transaction_hash(&data);

			let recoverd_pk = crypto::secp256k1_ecdsa_recover(&data.signature, &tx_hash_bytes).map_err(|_| VerifyError::SignatureError)?;

			if recoverd_pk != data.from {
				return Err(VerifyError::SignerNotCaller);
			}
	
			// Check nonce
			if nonce == data.nonce {
				// Add to transaction recorder
				let omni_tx = OmniverseTx::new(data.clone());
				tr.push(omni_tx);
				TransactionRecorder::<T>::insert(&data.from, tr);
				Ok(VerifyResult::Success)
			}
			else if nonce > data.nonce {
				// Check conflicts
				let his_tx = &tr[data.nonce as usize];
				let his_tx_hash = get_transaction_hash(&his_tx.tx_data);
				if his_tx_hash != tx_hash_bytes {
					let omni_tx = OmniverseTx::new(data.clone());
					let evil_tx = EvilTxData::new(omni_tx, nonce);
					let mut er = EvilRecorder::<T>::get(&data.from).unwrap_or(Vec::<EvilTxData>::default());
					er.push(evil_tx);
					EvilRecorder::<T>::insert(&data.from, er);
					Ok(VerifyResult::Malicious)
				}
				else {
					Ok(VerifyResult::Duplicated)
				}
			}
			else {
				Err(VerifyError::NonceError)
			}
		}
	
		fn get_transaction_count(pk: [u8; 64]) -> u128 {
			let record = Self::transaction_recorder(pk);
			if let Some(r) = record {
				return r.len() as u128;
			}
	
			0
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
			CHAIN_ID
		}
	}
}