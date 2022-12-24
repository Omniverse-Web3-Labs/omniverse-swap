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
	use omniverse_protocol_traits::{OmniverseAccounts, OmniverseTokenProtocol};
	use omniverse_token_traits::{OmniverseTokenFactoryHandler};

	const DEPOSIT: u8 = 0_u8;
	const TRANSFER: u8 = 1_u8;
	const WITHDRAW: u8 = 2_u8;
	const MINT: u8 = 3_u8;

    /// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type OmniverseProtocol: OmniverseAccounts;
	}

    #[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

    // The pallet's runtime storage items.
	// https://docs.substrate.io/v3/runtime/storage
	#[pallet::storage]
	#[pallet::getter(fn tokens_info)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/v3/runtime/storage#declaring-storage-items
	pub type TokensInfo<T:Config> = StorageMap<_, Blake2_128Concat, Vec<u8>, OmniverseToken<T::AccountId>>;

	#[pallet::storage]
	#[pallet::getter(fn tokens)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/v3/runtime/storage#declaring-storage-items
	pub type Tokens<T:Config> = StorageDoubleMap<_, Blake2_128Concat, Vec<u8>, Blake2_128Concat, Vec<u8>, u128>;

    // Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		TokenCreated(T::AccountId, Vec<u8>),
		TransactionSent(Vec<u8>, [u8; 64]),
		MembersSet(Vec<u8>, Vec::<u8>),
	}

    // Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		TokenAlreadyExist,
		/// Errors should have helpful documentation associated with them.
		TokenNotExist,
		NotOwner,
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
		pub fn create_token(origin: OriginFor<T>, token_id: Vec<u8>, members: Option<Vec<u8>>) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			// Check if the token exists
			ensure!(!TokensInfo::<T>::contains_key(&token_id), Error::<T>::TokenAlreadyExist);

			// Update storage.
			TokensInfo::<T>::insert(
                &token_id,
                OmniverseToken::new(sender.clone(), token_id.clone(), members)
            );

			// Emit an event.
			Self::deposit_event(Event::TokenCreated(sender, token_id));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		#[pallet::weight(0)]
		pub fn send_transaction(origin: OriginFor<T>, token_id: Vec<u8>, data: OmniverseTokenProtocol) -> DispatchResult {
			let sender = ensure_signed(origin)?;

            // Check if the token exists.
            let mut token = TokensInfo::<T>::get(&token_id).ok_or(Error::<T>::TokenNotExist)?;

            token.handle_transaction::<T>(&data);

            Self::deposit_event(Event::TransactionSent(token_id, data.from));

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn set_members(origin: OriginFor<T>, token_id: Vec<u8>, members: Vec<u8>) -> DispatchResult {
			let sender = ensure_signed(origin)?;

            // Check if the token exists.
            let mut token = TokensInfo::<T>::get(&token_id).ok_or(Error::<T>::TokenNotExist)?;

            ensure!(token.owner == sender, Error::<T>::NotOwner);

			token.add_members(members.clone());

            // Update storage
			TokensInfo::<T>::insert(&token_id, token);

            Self::deposit_event(Event::MembersSet(token_id, members));

			Ok(())
		}
	}

	#[derive(Decode, Encode)]
	pub struct TokenOpcode {
		op: u8,
		data: Vec<u8>
	}

	#[derive(Decode, Encode)]
	pub struct MintTokenOp {
		to: Vec<u8>,
		amount: u128
	}

	#[derive(Decode, Encode)]
	pub struct TransferTokenOp {
		to: Vec<u8>,
		amount: u128
	}

	#[derive(Clone, PartialEq, Eq, Debug, Encode, Decode, TypeInfo)]
	pub struct OmniverseToken<AccountId> {
		owner: AccountId,
		token_id: Vec<u8>,
		members: Vec<u8>
	}

	impl<AccountId> OmniverseToken<AccountId> {		
		fn new(owner: AccountId, token_id: Vec<u8>, members: Option<Vec<u8>>) -> Self {
			Self {
				owner,
				token_id,
				members: members.unwrap_or(Vec::<u8>::new())
			}
		}
		
		fn handle_transaction<T: Config>(&mut self, data: &OmniverseTokenProtocol) {
			// Check if the tx destination is correct
			assert!(data.to == self.token_id,
			"Wrong destination");
	
			// Check if the sender is honest
			assert!(!T::OmniverseProtocol::is_malicious(data.from), "User is malicious");
	
			// Verify the signature
			let ret = T::OmniverseProtocol::verify_transaction(&data);
			assert!(ret.is_ok());
	
			// Execute
			let op_data = TokenOpcode::decode(&mut data.data.as_slice()).unwrap();
			if op_data.op == DEPOSIT {
	
			}
			else if op_data.op == TRANSFER {
				let transfer_data = TransferTokenOp::decode(&mut op_data.data.as_slice()).unwrap();
				self.omniverse_transfer(transfer_data.to, transfer_data.amount);
			}
			else if op_data.op == WITHDRAW {
	
			}
			else if op_data.op == MINT {
				let mint_data = TransferTokenOp::decode(&mut op_data.data.as_slice()).unwrap();
				self.omniverse_mint(mint_data.to, mint_data.amount);
			}
		}
	
		fn omniverse_transfer(&mut self, to: Vec<u8>, amount: u128) {
	
		}
	
		fn omniverse_mint(&mut self, to: Vec<u8>, amount: u128) {
	
		}
	
		fn add_members(&mut self, members: Vec<u8>) {
			for m in &members {
				if !self.members.contains(m) {
					self.members.push(*m)
				}
			}
		}
	
		fn get_members(&self) -> Vec<u8> {
			self.members.clone()
		}
	}

	pub struct OmniverseTokenFactory<T>(T);

	impl<T: Config> OmniverseTokenFactoryHandler for OmniverseTokenFactory<T> {
		fn send_transaction(&mut self, token_id: Vec<u8>, data: &OmniverseTokenProtocol) -> Result<(), ()> {
			// Check if the token exists.
            let mut token = TokensInfo::<T>::get(&token_id).ok_or(())?;

            token.handle_transaction::<T>(&data);

			Ok(())
		}
	}
}