#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

use parity_scale_codec::{Encode, Decode};

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

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use sp_std::vec::Vec;
	use parity_scale_codec::{Encode, Decode};
	use omniverse_protocol::{OmniverseTokenProtocol, OmniverseAccounts};

	const DEPOSIT = 0_u8;
	const TRANSFER = 1_u8;
	const WITHDRAW = 2_u8;
	const MINT = 3_u8;

    /// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type OmniverseProtocol: OmniverseAccounts;
	}

    #[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

    // The pallet's runtime storage items.
	// https://docs.substrate.io/v3/runtime/storage
	#[pallet::storage]
	#[pallet::getter(fn tokens)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/v3/runtime/storage#declaring-storage-items
	pub type Tokens<T:Config> = StorageMap<_, Blake2_128Concat, String, OmniverseToken>;

    // Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		TokenCreated(T::AccountId, String),
		TransactionSent(String, Vec::<u8>),
		MembersSet(String, Vec::<u8>),
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
		pub fn create_token(origin: OriginFor<T>, token_id: String, members: Option<Vec<u8>>) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			// Check if the token exists
			ensure!(!Tokens::<T>::contains_key(&token_id), Error::<T>::TokenAlreadyExist);

			// Update storage.
			<Tokens<T>>::insert(
                &token_id,
                OmniverseToken::new(sender, token_id, members)
            );

			// Emit an event.
			Self::deposit_event(Event::TokenCreated(sender, token_id));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		#[pallet::weight(0)]
		pub fn send_transaction(origin: OriginFor<T>, token_id: String, data: Vec<u8>) -> DispatchResult {
			let sender = ensure_signed(origin)?;

            // Check if the token exists.
            let token = Tokens::<T>::get(&token_id).ok_or(Error::<T>::TokenNotExist)?;

            token.handle_transaction(data);

            // Update storage
			Tokens::<T>::insert(&token_id, token);

            Self::deposit_event(Event::TransactionSent(token_id, data.from));

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn set_members(origin: OriginFor<T>, token_id: String, members: Vec<Vec<u8>>) -> DispatchResult {
			let sender = ensure_signed(origin)?;

            // Check if the token exists.
            let token = Tokens::<T>::get(&token_id).ok_or(Error::<T>::TokenNotExist)?;

            ensure!(token.owner == sender, Error::<T>::NotOwner);

			token.set_members(members);

            // Update storage
			Tokens::<T>::insert(&token_id, token);

            Self::deposit_event(Event::MembersSet(token_id, members));

			Ok(())
		}
	}
}

pub struct OmniverseToken {
	owner: Vec<u8>,
	token_id: String,
	members: Vec<u8>,
	omniverse_balances: HashMap<Vec<u8>, u128>;
}

impl OmniverseToken {
	fn new(owner: Vec<u8>, token_id: String, members: Option<Vec<u8>>) -> Self {
		Self {
			owner,
			token_id,
			members: members.unwrap_or(Vec<u8>::new()),
			omniverse_balances: HashMap<Vec<u8>, u128>::new
		}
	}
}

impl<T: Config> OmniverseToken {
	fn handle_transaction(&mut self, data: OmniverseTokenProtocol) {
		// Check if the tx destination is correct
		assert!(data.to == OmniverseProtocol::<T>::get_chain_id(),
		"Wrong destination");

		// Check if the sender is honest
		assert!(!OmniverseProtocol::<T>::is_malicious(data.from), "User is malicious");

		// Verify the signature
		let ret = OmniverseProtocol::<T>::verify_transaction(data);
		assert!(ret.is_ok());

		// Execute
		let op_data = TokenOpcode::decode(data.data);
		if op_data.op == DEPOSIT {

		}
		else op == TRANSFER {
			let transfer_data = TransferTokenOp::decode(op_data.data);
			self.omniverse_transfer(transfer_data.to, transfer_data.amount);
		}
		else op == WITHDRAW {

		}
		else op == MINT {
			let mint_data = TransferTokenOp::decode(op_data.data);
			self.omniverse_mint(mint_data.to, mint_data.amount);
		}
	}

	fn omniverse_transfer(&mut self, to: Vec<u8>, amount: u128) {

	}

	fn omniverse_mint(&mut self, to: Vec<u8>, amount: u128) {

	}

	fn omniverse_balance_of(&self, user: Vec<u8>) -> u128 {
		self.omniverse_balances.get(&user).unwrap_or(0);
	}

	fn add_members(&mut self, members: Vec<String>) {
		for m in &members {
			if !self.members.contains(m) {
				self.members.push(m)
			}
		}
	}

	fn get_members(&self) -> Vec<String> {
		self.members
	}
}