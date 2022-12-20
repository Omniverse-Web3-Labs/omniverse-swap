#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
use std::time::SystemTime;
use sha3::{Digest, Keccak256};
use Keccak256::{Secp256k1, Message, ecdsa, PublicKey};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[derive(Clone, PartialEq, Eq, Debug, TypeInfo)]
pub struct OmniverseTokenProtocol {
	nonce: u128,
	chain_id: u8,
	from: Vec<u8>,
	to: String,
	data: Vec<u8>,
	signature: Vec<u8>
};

#[derive(Clone, PartialEq, Eq, Debug, TypeInfo)]
pub struct OmniverseTx {
	tx_data: OmniverseTokenProtocol,
	timestamp: u128
};

impl OmniverseTx {
	fn new(data: OmniverseTokenProtocol) -> Self {
		Self {
			tx_data: data,
			timestamp: SystemTime::now().elapsed().as_secs()
		}
	}
}

#[derive(Clone, PartialEq, Eq, Debug, TypeInfo)]
pub struct EvilTxData {
	tx_omni: OmniverseTx,
	his_nonce: u128
};

impl EvilTxData {
	fn new (data: OmniverseTx, nonce: u128) -> Self {
		Self {
			tx_omni: data,
			his_nonce: nonce
		}
	}
}

#[derive(Clone, PartialEq, Eq, Debug, TypeInfo)]
pub struct RecordedCertificate {
	tx_list: Vec<OmniverseTx>,
	evil_tx_list: Vec<EvilTxData>
};

#[derive(Clone, PartialEq, Eq, Debug, TypeInfo)]
pub enum VerifyResult {
	Success,
	Malicious,
	Duplicated
};

#[derive(Clone, PartialEq, Eq, Debug, TypeInfo)]
pub enum VerifyError {
	SignatureError,
	NonceError
};

pub trait OmniverseAccounts {
	fn verify_transaction(data: OmniverseTokenProtocol) -> VerifyResult;
	fn get_transaction_count(pk: Vec<u8>) -> u128;
	fn is_malicious(pk: Vec<u8>) -> bool;
	fn get_transaction_data(pk: Vec<u8>, nonce: u128) -> OmniverseTokenProtocol;
	fn get_chain_id() -> String;
}

#[frame_support::pallet]
pub mod pallet {    
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use sp_std::vec::Vec;

	#[pallet::type_value]
	pub fn GetDefaultChainId<T: Config>() -> T::ChainId {
		"".into()
	}

	#[pallet::type_value]
	pub fn GetDefaultCDTime<T: Config>() -> T::CDTime {
		0_u32.into()
	}

    /// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type ChainId: Parameter
			+ Member
			+ AtLeast32Bit
			+ Codec
			+ Default
			+ Copy
			+ MaybeSerializeDeserialize
			+ Debug
			+ MaxEncodedLen
			+ TypeInfo;
		type CDTime: Parameter
			+ Member
			+ AtLeast32Bit
			+ Codec
			+ Default
			+ Copy
			+ MaybeSerializeDeserialize
			+ Debug
			+ MaxEncodedLen
			+ TypeInfo;
	}

    #[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

    // The pallet's runtime storage items.
	// https://docs.substrate.io/v3/runtime/storage
	#[pallet::storage]
	#[pallet::getter(fn transaction_recorder)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/v3/runtime/storage#declaring-storage-items
	pub type TransactionRecorder<T:Config> = StorageMap<_, Blake2_128Concat, Vec<u8>, RecordedCertificate>;

	#[pallet::storage]
	#[pallet::getter(fn chain_id)]
	pub type ChainId<T:Config> = StorageValue<_, T::ChainId, ValueQuery, GetDefaultChainId<T>>;

	#[pallet::storage]
	#[pallet::getter(fn cd_time)]
	pub type CDTime<T:Config> = StorageValue<_, T::CDTime, ValueQuery, GetDefaultCDTime<T>>;

    // Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		ClaimCreated(T::AccountId, Vec::<u8>),
		ClaimRevoked(T::AccountId, Vec::<u8>),
		ClaimTransferred(T::AccountId, T::AccountId, Vec::<u8>),
	}

    // Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		ClaimAlreadyExist,
		/// Errors should have helpful documentation associated with them.
		ClaimNotExist,
		/// There have be doc.
		OnlyOwnerCanRevoke,
		NotAbleToTransferToSelf,
		OnlyOwnerCanTransfer,
		ClaimLengthError,
	}

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    // Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[pallet::weight(0)]
		pub fn verify_transaction(origin: OriginFor<T>, data: OmniverseTokenProtocol) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			ensure!(claim.len() == 8, Error::<T>::ClaimLengthError);

            // Check if the claim exists.
            ensure!(!Proofs::<T>::contains_key(&claim), Error::<T>::ClaimAlreadyExist);

			// Update storage.
			<Proofs<T>>::insert(
                &claim,
                (sender.clone(), frame_system::Pallet::<T>::block_number())
            );

			// Emit an event.
			Self::deposit_event(Event::ClaimCreated(sender, claim));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}
	}
}

fn get_transaction_hash(data: OmniverseTokenProtocol) -> Vec<u8> {
	let mut raw = Vec<u8>::new();
	raw.append(u128::to_be_bytes(data.nonce));
	raw.append(u8::to_be_bytes(data.chain_id));
	raw.append(data.from);
	raw.append(data.to.as_mut_vec());
	raw.append(data.data);

	let mut hasher = Keccak256::new();
    hasher.update(data);
    let result = hasher.finalize();

    result.to_vec()
}

impl<T: Config> OmniverseAccounts for Pallet<T> {
	fn verify_transaction(data: OmniverseTokenProtocol) -> Result<VerifyResult, VerifyError> {
		let mut rc = TransactionRecorder::<T>::get(&data.from).unwrap_or(RecordedCertificate::default());
		u128 nonce = rc.tx_list.length;

		let tx_hash_bytes = get_transaction_hash(data);
		let hash_message = Message::from_slice(tx_hash_bytes.as_ref()).expect("messages must be 32 bytes and are expected to be hashes");
		let sig = ecdsa::Signature::from_compact(data.signature.as_ref()).expect("compact signatures are 64 bytes; DER signatures are 68-72 bytes");
		let secp = Secp256k1::new();
		if !secp.verify_ecdsa(&hash_message, &sig, &data.from).is_ok() {
			return Err(VerifyError::SignatureError);
		}

		// Check nonce
		if nonce == data.nonce {
			// Add to transaction recorder
			let omni_tx = OmniverseTx::new(data);
			rc.tx_list.push(omni_tx);
			RecordedCertificate::<T>::insert(&data.from, rc);
			Ok(VerifyResult::Success)
		}
		else if nonce > data.nonce {
			// Check conflicts
			let his_tx = rc.tx_list[_data.nonce];
			let his_tx_hash = get_transaction_hash(his_tx.tx_data);
			if his_tx_hash != hash_message {
				let omni_tx = OmniverseTx::new(data);
				let evil_tx = EvilTxData::new(omni_tx, nonce);
				rc.evil_tx_list.push(evil_tx);
				RecordedCertificate::<T>::insert(&data.from, rc);
				Ok(VerifyResult::Malicious)
			}
			else {
				Ok(VerifyResult::Duplicated)
			}
		}
		else {
			Err(VerifyError::VerifyError)
		}
	}

	fn get_transaction_count(pk: Vec<u8>) -> u128 {
		let record = Self::transaction_recorder(pk);
		if let Some(r) = record {
			return r.tx_list.length;
		}

		0
	}

	fn is_malicious(pk: Vec<u8>) -> bool {
		let record = Self::transaction_recorder(pk);
		if let Some(r) = record {
			if r.evil_tx_list.length > 0 {
				return true;
			}
		}

		false
	}

	fn get_transaction_data(pk: Vec<u8>, nonce: u128) -> Option<OmniverseTokenProtocol> {
		let record = Self::transaction_recorder(pk);
		if let Some(r) = record {
			r.tx_list.get(nonce);
		}

		None
	}

	fn get_chain_id() -> String {
		Self::chain_id().unwrap()
	}
}