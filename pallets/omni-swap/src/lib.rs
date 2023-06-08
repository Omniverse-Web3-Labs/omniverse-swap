#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

// current support assets
// pub static PALLET_NAME: [u8; 6] = [0x61, 0x73, 0x73, 0x65, 0x74, 0x73];
#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use sp_std::vec::Vec;
	// use sp_runtime::traits::TrailingZeroInput;
	use pallet_assets::{traits::OmniverseTokenFactoryHandler, PALLET_NAME};
	use pallet_omniverse_protocol::{
		traits::OmniverseAccounts, Fungible, OmniverseTransactionData,
	};
	use secp256k1::PublicKey;
	use sp_core::Hasher;
	use sp_runtime::traits::BlakeTwo256;
	use sp_runtime::traits::IntegerSquareRoot;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);
	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type OmniverseProtocol: OmniverseAccounts;
		type OmniverseToken: OmniverseTokenFactoryHandler;
	}

	#[pallet::storage]
	#[pallet::getter(fn trading_pairs)]
	pub type TradingPairs<T: Config> = StorageMap<_, Blake2_128Concat, Vec<u8>, (u128, u128)>;

	#[pallet::storage]
	#[pallet::getter(fn total_liquidity)]
	pub type TotalLiquidity<T: Config> = StorageMap<_, Blake2_128Concat, Vec<u8>, u128>;

	#[pallet::storage]
	#[pallet::getter(fn liquidity)]
	pub type Liquidity<T: Config> = StorageMap<_, Blake2_128Concat, (Vec<u8>, [u8; 64]), u128>;

	// #[pallet::storage]
	// #[pallet::getter(fn balance)]
	// pub type Balance<T: Config> =
	// 	StorageMap<_, Blake2_128Concat, (Vec<u8>, [u8; 64]), (u128, u128)>;

	#[pallet::storage]
	#[pallet::getter(fn token_id)]
	pub type TokenId<T: Config> = StorageMap<_, Blake2_128Concat, Vec<u8>, (Vec<u8>, Vec<u8>)>;
	// #[pallet::storage]
	// #[pallet::getter(fn public_key)]
	// pub type PublicKey<T:Config> = StorageMap<_, Blake2_128Concat, T::AccountId, [u8; 64]>;

	#[pallet::storage]
	#[pallet::getter(fn deposit_record)]
	pub type DepositRecords<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		[u8; 64],
		Blake2_128Concat,
		(Vec<u8>, u128),
		OmniverseTransactionData,
	>;

	/// key: pk and token_id
	/// value: balance
	#[pallet::storage]
	#[pallet::getter(fn balance)]
	pub type Balance<T: Config> =
		StorageDoubleMap<_, Blake2_128Concat, [u8; 64], Blake2_128Concat, Vec<u8>, u128>;

	/// key: pk
	/// value: withdraw amount
	#[pallet::storage]
	#[pallet::getter(fn withdrawals)]
	pub type Withdrawals<T: Config> =
		StorageDoubleMap<_, Blake2_128Concat, [u8; 64], Blake2_128Concat, Vec<u8>, u128>;

	#[pallet::storage]
	#[pallet::getter(fn mpc)]
	pub type Mpc<T: Config> = StorageValue<_, [u8; 64], ValueQuery, GetDefaultMpc>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		SwapX2YTokens(Vec<u8>, [u8; 64], u128, u128),
		SwapY2XTokens(Vec<u8>, [u8; 64], u128, u128),
		AddLiquidity(Vec<u8>, [u8; 64], u128, u128),
		RemoveLiquidity(Vec<u8>, [u8; 64], u128, u128),
		/// public_key, token_id, nonce
		PendingDeposit([u8; 64], Vec<u8>, u128),
		/// public_key, token_id, nonce
		DepositComfirmed([u8; 64], Vec<u8>, u128),
		/// public_key, token_id, amount
		Withdrawal([u8; 64], Vec<u8>, u128),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		DecodePayloadFailed,
		SerializePublicKeyFailed,
		StorageOverflow,
		InvalidValue,
		TradingPairNotExist,
		InsufficientBAmount,
		InsufficientAAmount,
		ExceedDesiredAmount,
		GetAddress0Failed,
		InsufficientLiquidity,
		InsufficientAmount,
		OmniverseTransferXFailed,
		OmniverseTransferYFailed,
		TokenIdNotExist,
		MismatchTokenId,
		InsufficientBalance,
		NotOmniverseTransfer,
		GetXTokenLessThenDesired,
		GetYTokenLessThenDesired,
		PublicKeyNotExist,
		MismatchReceiptor,
		DepositExist,
		NotDeposit,
		IsComfirmed,
		TxNotExisted,
		/// Deposit tx mismatch record tx
		OmniverseTxMismatch,
		BalanceNotEnough,

		/// Check permission
		NoPermission,
		ToAccountMismatch,
		///
		WithdrawalNotExist,
		WithdrawAmountMismatch,
	}

	/// for default mpc account
	#[pallet::type_value]
	pub fn GetDefaultMpc() -> [u8; 64] {
		[
			155, 11, 196, 48, 165, 127, 191, 171, 40, 110, 174, 255, 210, 45, 31, 5, 188, 65, 92,
			111, 60, 25, 212, 196, 136, 12, 62, 31, 128, 229, 167, 166, 94, 54, 163, 249, 96, 173,
			218, 70, 112, 166, 144, 9, 138, 16, 173, 152, 240, 49, 17, 212, 8, 90, 147, 115, 37,
			170, 70, 128, 114, 220, 242, 148,
		]
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Deposit
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn deposit(
			origin: OriginFor<T>,
			token_id: Vec<u8>,
			data: OmniverseTransactionData,
		) -> DispatchResult {
			ensure_signed(origin)?;
			// Transfer X token to MPC account
			let mpc = Mpc::<T>::get();
			let fungible = Fungible::decode(&mut data.payload.as_slice())
				.map_err(|_| Error::<T>::DecodePayloadFailed)?;
			let to: [u8; 64] =
				fungible.ex_data.try_into().map_err(|_| Error::<T>::SerializePublicKeyFailed)?;
			ensure!(to == mpc, Error::<T>::InvalidValue);
			T::OmniverseToken::send_transaction_external(token_id.clone(), &data)
				.ok()
				.ok_or(Error::<T>::OmniverseTransferXFailed)?;
			// let omni_tx = OmniverseTx::new(data.clone(), T::Timestamp::now().as_secs());
			ensure!(
				!DepositRecords::<T>::contains_key(data.from, &(token_id.clone(), data.nonce)),
				Error::<T>::DepositExist
			);
			DepositRecords::<T>::insert(data.from, &(token_id.clone(), data.nonce), data.clone());
			Self::deposit_event(Event::PendingDeposit(data.from, token_id, data.nonce));
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn withdraw(
			origin: OriginFor<T>,
			pk: [u8; 64],
			token_id: Vec<u8>,
			amount: u128,
		) -> DispatchResult {
			// TODO: Is it necessary to verify whether "from" is an MPC account? Currently, it is open to anyone.
			let sender = ensure_signed(origin)?;
			let owner = Self::to_account(&pk)?;
			ensure!(sender == owner, Error::<T>::NoPermission);

			let balance = Balance::<T>::get(pk, &token_id).unwrap_or(0);
			ensure!(amount > 0 && balance >= amount, Error::<T>::InvalidValue);
			Withdrawals::<T>::insert(pk, &token_id, amount);
			Balance::<T>::insert(pk, &token_id, balance - amount);

			Self::deposit_event(Event::Withdrawal(pk, token_id, amount));
			Ok(())
		}

		/// Once the omniverse transaction has been executed, any account is
		/// eligible to initiate the conclusive confirmation of the final deposit.
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn deposit_comfirm(
			origin: OriginFor<T>,
			pk: [u8; 64],
			token_id: Vec<u8>,
			nonce: u128,
		) -> DispatchResult {
			ensure_signed(origin)?;
			let data = DepositRecords::<T>::get(pk, &(token_id.clone(), nonce))
				.ok_or(Error::<T>::NotDeposit)?;
			let omni_tx = T::OmniverseProtocol::get_transaction_data(
				pk,
				PALLET_NAME.to_vec(),
				token_id.clone(),
				nonce,
			)
			.ok_or(Error::<T>::TxNotExisted)?;

			ensure!(data == omni_tx.tx_data, Error::<T>::OmniverseTxMismatch);

			DepositRecords::<T>::remove(pk, &(token_id.clone(), nonce));
			// let balance
			let mut balance = Balance::<T>::get(pk, &token_id).unwrap_or(0);
			let fungible = Fungible::decode(&mut data.payload.as_slice())
				.map_err(|_| Error::<T>::DecodePayloadFailed)?;
			balance += fungible.amount;
			Balance::<T>::insert(pk, &token_id, balance);
			Self::deposit_event(Event::DepositComfirmed(data.from, token_id, data.nonce));
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn withdraw_comfirm(
			origin: OriginFor<T>,
			pk: [u8; 64],
			token_id: Vec<u8>,
			data: OmniverseTransactionData,
		) -> DispatchResult {
			ensure_signed(origin)?;
			let withdrawal =
				Withdrawals::<T>::get(pk, &token_id).ok_or(Error::<T>::WithdrawalNotExist)?;
			let fungible = Fungible::decode(&mut data.payload.as_slice())
				.map_err(|_| Error::<T>::DecodePayloadFailed)?;
			ensure!(withdrawal == fungible.amount, Error::<T>::WithdrawAmountMismatch);
			let dest_pk: [u8; 64] =
				fungible.ex_data.try_into().map_err(|_| Error::<T>::SerializePublicKeyFailed)?;
			ensure!(pk == dest_pk, Error::<T>::ToAccountMismatch);

			Withdrawals::<T>::remove(pk, &token_id);
			T::OmniverseToken::send_transaction_external(token_id, &data)
				.ok()
				.ok_or(Error::<T>::OmniverseTransferYFailed)?;
			Ok(())
		}

		/// Convert X token to Y token
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn swap_x2y(
			origin: OriginFor<T>,
			trading_pair: Vec<u8>,
			pk: [u8; 64],
			tokens_sold: u128,
			min_token: u128,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let owner = Self::to_account(&pk)?;
			ensure!(sender == owner, Error::<T>::NoPermission);
			ensure!(tokens_sold > 0 && min_token > 0, Error::<T>::InvalidValue);
			let (token_x_id, token_y_id) =
				TokenId::<T>::get(&trading_pair).ok_or(Error::<T>::TradingPairNotExist)?;
			let balance_x = Balance::<T>::get(pk, &token_x_id).unwrap_or(0);
			ensure!(balance_x >= tokens_sold, Error::<T>::BalanceNotEnough);

			let (reserve_x, reserve_y) =
				TradingPairs::<T>::get(&trading_pair).ok_or(Error::<T>::TradingPairNotExist)?;
			let tokens_bought: u128 = get_input_price(tokens_sold, reserve_x, reserve_y);
			ensure!(tokens_bought >= min_token, Error::<T>::GetYTokenLessThenDesired);
			<TradingPairs<T>>::insert(
				&trading_pair,
				(reserve_x + tokens_sold, reserve_y - tokens_bought),
			);

			// update token_x and token_y balance
			let balance_y = Balance::<T>::get(pk, &token_y_id).unwrap_or(0);
			Balance::<T>::insert(pk, &token_x_id, balance_x - tokens_sold);
			Balance::<T>::insert(pk, &token_y_id, balance_y + tokens_bought);

			Self::deposit_event(Event::SwapX2YTokens(trading_pair, pk, tokens_sold, tokens_bought));
			Ok(())
		}

		/// Convert Y token to X token
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn swap_y2x(
			origin: OriginFor<T>,
			trading_pair: Vec<u8>,
			pk: [u8; 64],
			tokens_sold: u128,
			min_token: u128,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let owner = Self::to_account(&pk)?;
			ensure!(sender == owner, Error::<T>::NoPermission);
			ensure!(tokens_sold > 0 && min_token > 0, Error::<T>::InvalidValue);
			let (token_x_id, token_y_id) =
				TokenId::<T>::get(&trading_pair).ok_or(Error::<T>::TradingPairNotExist)?;
			let balance_y = Balance::<T>::get(pk, &token_y_id).unwrap_or(0);
			ensure!(balance_y >= tokens_sold, Error::<T>::BalanceNotEnough);

			let (reserve_x, reserve_y) =
				TradingPairs::<T>::get(&trading_pair).ok_or(Error::<T>::TradingPairNotExist)?;
			let tokens_bought = get_input_price(tokens_sold, reserve_y, reserve_x);
			ensure!(tokens_bought >= min_token, Error::<T>::GetXTokenLessThenDesired);
			<TradingPairs<T>>::insert(
				&trading_pair,
				(reserve_x - tokens_bought, reserve_y + tokens_sold),
			);

			// update token_x and token_y balance
			let balance_x = Balance::<T>::get(pk, &token_x_id).unwrap_or(0);
			Balance::<T>::insert(pk, &token_x_id, balance_x + tokens_bought);
			Balance::<T>::insert(pk, &token_y_id, balance_y - tokens_sold);

			Self::deposit_event(Event::SwapY2XTokens(trading_pair, pk, tokens_sold, tokens_bought));
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn add_liquidity(
			origin: OriginFor<T>,
			trading_pair: Vec<u8>,
			pk: [u8; 64],
			amount_x_desired: u128,
			amount_y_desired: u128,
			amount_x_min: u128,
			amount_y_min: u128,
			token_x_id: Vec<u8>,
			token_y_id: Vec<u8>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let owner = Self::to_account(&pk)?;
			ensure!(sender == owner, Error::<T>::NoPermission);
			ensure!(amount_x_desired > 0 && amount_y_desired > 0, Error::<T>::InvalidValue);

			if !TokenId::<T>::contains_key(&trading_pair) {
				<TokenId<T>>::insert(&trading_pair, (token_x_id.clone(), token_y_id.clone()));
			}

			let tranding_pair = TradingPairs::<T>::get(&trading_pair);
			let amount_x: u128;
			let amount_y: u128;
			if tranding_pair.is_some() {
				let (reserve_x, reserve_y) =
					TradingPairs::<T>::get(&trading_pair).ok_or(Error::<T>::TradingPairNotExist)?;
				let amount_y_optimal = quote(amount_x_desired, reserve_x, reserve_y);
				if amount_y_optimal <= amount_y_desired {
					ensure!(
						amount_y_optimal > 0 && amount_y_min > 0,
						Error::<T>::InsufficientBAmount
					);
					amount_x = amount_x_desired;
					amount_y = amount_y_optimal;
				} else {
					let amount_x_optimal = quote(amount_y_desired, reserve_y, reserve_x);
					ensure!(amount_x_optimal <= amount_x_desired, Error::<T>::ExceedDesiredAmount);
					ensure!(
						amount_x_optimal > 0 && amount_x_min > 0,
						Error::<T>::InsufficientAAmount
					);
					amount_x = amount_x_optimal;
					amount_y = amount_y_desired;
				}
				<TradingPairs<T>>::insert(
					&trading_pair,
					(reserve_x + amount_x, reserve_y + amount_y),
				);
			} else {
				amount_x = amount_x_desired;
				amount_y = amount_y_desired;
				<TradingPairs<T>>::insert(&trading_pair, (amount_x, amount_y));
				<TotalLiquidity<T>>::insert(&trading_pair, 0u128);
			}

			let balance_x = Balance::<T>::get(pk, &token_x_id).unwrap_or(0);
			let balance_y = Balance::<T>::get(pk, &token_y_id).unwrap_or(0);
			ensure!(
				balance_x >= amount_x && balance_y >= amount_y,
				Error::<T>::InsufficientBalance
			);

			Balance::<T>::insert(pk, &token_x_id, balance_x - amount_x);
			Balance::<T>::insert(pk, &token_y_id, balance_y - amount_y);

			let key = (trading_pair.clone(), pk);
			// mint
			let (balance_x, balance_y) =
				TradingPairs::<T>::get(&trading_pair).ok_or(Error::<T>::TradingPairNotExist)?;
			let mut total_supply =
				TotalLiquidity::<T>::get(&trading_pair).ok_or(Error::<T>::TradingPairNotExist)?;
			let liquidity: u128;
			if total_supply == 0 {
				liquidity = (amount_x * amount_y).integer_sqrt().saturating_sub(1000);
				total_supply = liquidity;
			} else {
				// liquidity = Math.min(amount0.mul(_totalSupply) / _reserve0, amount1.mul(_totalSupply) / _reserve1);
				liquidity = (amount_x.saturating_mul(total_supply) / (balance_x - amount_x))
					.min(amount_y.saturating_mul(total_supply) / (balance_y - amount_y));
				total_supply += liquidity;
			}
			let balances = Liquidity::<T>::get(&key).unwrap_or(0) + liquidity;
			<Liquidity<T>>::insert(&key, balances);
			<TotalLiquidity<T>>::insert(&trading_pair, total_supply);

			Self::deposit_event(Event::AddLiquidity(trading_pair, pk, amount_x, amount_y));
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn remove_liquidity(
			origin: OriginFor<T>,
			trading_pair: Vec<u8>,
			pk: [u8; 64],
			liquidity: u128,
			amount_x_min: u128,
			amount_y_min: u128,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let owner = Self::to_account(&pk)?;
			ensure!(sender == owner, Error::<T>::NoPermission);

			let key = (trading_pair.clone(), pk);
			let balances = Liquidity::<T>::get(&key).unwrap_or(0);
			ensure!(balances >= liquidity, Error::<T>::InvalidValue);

			// burn
			let (reserve_x, reserve_y) =
				TradingPairs::<T>::get(&trading_pair).ok_or(Error::<T>::TradingPairNotExist)?;
			<Liquidity<T>>::insert(&key, balances - liquidity);
			let total_supply =
				TotalLiquidity::<T>::get(&trading_pair).ok_or(Error::<T>::TradingPairNotExist)?;
			let amount_x = liquidity.saturating_mul(reserve_x) / total_supply;
			let amount_y = liquidity.saturating_mul(reserve_y) / total_supply;
			ensure!(
				amount_x >= amount_x_min && amount_y >= amount_y_min,
				Error::<T>::InsufficientAmount
			);

			<TotalLiquidity<T>>::insert(&trading_pair, total_supply - liquidity);
			<TradingPairs<T>>::insert(&trading_pair, (reserve_x - amount_x, reserve_y - amount_y));

			let (token_x_id, token_y_id) =
				TokenId::<T>::get(&trading_pair).ok_or(Error::<T>::TradingPairNotExist)?;
			let balance_x = Balance::<T>::get(pk, &token_x_id).unwrap_or(0);
			let balance_y = Balance::<T>::get(pk, &token_y_id).unwrap_or(0);

			Balance::<T>::insert(pk, &token_x_id, balance_x + amount_x);
			Balance::<T>::insert(pk, &token_y_id, balance_y + amount_y);
			Self::deposit_event(Event::RemoveLiquidity(trading_pair, pk, amount_x, amount_y));
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn set_mpc(origin: OriginFor<T>, new_mpc: [u8; 64]) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let mpc = Self::to_account(&new_mpc)?;
			ensure!(mpc == sender, Error::<T>::NoPermission);
			Mpc::<T>::set(new_mpc);
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn to_account(public_key: &[u8; 64]) -> Result<T::AccountId, Error<T>> {
			let mut pk_full: [u8; 65] = [0; 65];
			pk_full[1..65].copy_from_slice(public_key);
			pk_full[0] = 4;
			let public_key = PublicKey::from_slice(&pk_full[..])
				.map_err(|_| Error::<T>::SerializePublicKeyFailed)?;
			let public_key_compressed = public_key.serialize();
			let hash = BlakeTwo256::hash(&public_key_compressed);
			Ok(T::AccountId::decode(&mut &hash[..]).unwrap())
		}
	}

	// impl<T: Config> Pallet<T> {
	pub fn get_input_price(input_amount: u128, input_reserve: u128, output_reserve: u128) -> u128 {
		// ensure!(input_reserve > 0 && output_reserve > 0u128);
		let numerator: u128 = input_amount * output_reserve;
		let denominator: u128 = input_reserve + input_amount;
		numerator / denominator
	}

	pub fn get_output_price(output_amout: u128, input_reserve: u128, output_reserve: u128) -> u128 {
		// ensure!(input_reserve > 0u128 && output_reserve > 0u128);
		let numerator: u128 = input_reserve * output_amout;
		let denominator: u128 = output_reserve - output_amout;
		numerator / denominator
	}

	/// given some amount of an asset and pair reserves, returns an equivalent amount of the other asset
	pub fn quote(amount_x: u128, reserve_x: u128, reserve_y: u128) -> u128 {
		amount_x * reserve_y / reserve_x
	}
	// }
}
