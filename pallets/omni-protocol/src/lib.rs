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

	#[pallet::type_value]
	pub fn GetDefaultChainId<T: Config>() -> T::ChainId {
		"".into()
	}

	#[pallet::type_value]
	pub fn GetDefaultCDTime<T: Config>() -> T::CDTime {
		0_u32.into()
	}

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

	#[derive(Clone, PartialEq, Eq, Debug, TypeInfo)]
	pub struct EvilTxData {
		tx_omni: OmniverseTx,
		his_nonce: u128
	};

	#[derive(Clone, PartialEq, Eq, Debug, TypeInfo)]
	pub struct RecordedCertificate {
		tx_list: Vec<OmniverseTx>,
		evil_tx_list: Vec<EvilTxData>
	};

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