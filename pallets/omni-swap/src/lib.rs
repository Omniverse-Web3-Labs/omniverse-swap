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

#[frame_support::pallet]
pub mod pallet {
	use sp_std::vec::Vec;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	// use sp_runtime::traits::TrailingZeroInput;
	use omniverse_protocol_traits::OmniverseTokenProtocol;
	use pallet_omniverse_factory::TRANSFER;
	use omniverse_token_traits::{OmniverseTokenFactoryHandler, TokenOpcode, TransferTokenOp};

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type OmniverseToken: OmniverseTokenFactoryHandler;
	}

	#[pallet::storage]
	#[pallet::getter(fn trading_pairs)]
	pub type TradingPairs<T:Config> = StorageMap<_, Blake2_128Concat, Vec<u8>, (u128, u128)>;

	#[pallet::storage]
	#[pallet::getter(fn total_liquidity)]
	pub type TotalLiquidity<T:Config> = StorageMap<_, Blake2_128Concat, Vec<u8>, u128>;

	#[pallet::storage]
	#[pallet::getter(fn liquidity)]
	pub type Liquidity<T:Config> = StorageMap<_, Blake2_128Concat, (Vec<u8>, T::AccountId), u128>;

	#[pallet::storage]
	#[pallet::getter(fn balance)]
	pub type Balance<T:Config> = StorageMap<_, Blake2_128Concat, (Vec<u8>, T::AccountId), (u128, u128)>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		// SomethingStored(u32, T::AccountId),
		TokenPurchase(Vec<u8>, T::AccountId, u128, u128),
		SwapAtoBTokens(Vec<u8>, T::AccountId, u128, u128),
		SwapBtoATokens(Vec<u8>, T::AccountId, u128, u128),
		AddLiquidity(Vec<u8>, T::AccountId, u128, u128),
		RemoveLiquidity(Vec<u8>, T::AccountId, u128, u128),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
		InvalidValue,
		TradingPairNotExist,
		InsufficientBAmount,
		InsufficientAAmount,
		ExceedDesiredAmount,
		GetAddress0Failed,
		InsufficientLiquidity,
		InsufficientAmount,
		OmniverseTransferFailed,
		NotOmniverseTransfer,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {

		/// Convert A token to B token
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn swap_atob(origin: OriginFor<T>, trading_pair: Vec<u8>, tokens_sold: u128, min_token: u128, token_a_id: Vec<u8>, token_a_data: OmniverseTokenProtocol) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			ensure!(tokens_sold > 0 && min_token > 0, Error::<T>::InvalidValue);
			// Transfer A token to MPC account
			T::OmniverseToken::send_transaction_external(token_a_id, &token_a_data).ok().ok_or(Error::<T>::OmniverseTransferFailed)?;

			let (input_reserve, out_reserve) = TradingPairs::<T>::get(&trading_pair).ok_or(Error::<T>::TradingPairNotExist)?;
			let tokens_bought: u128 = get_input_price(tokens_sold, input_reserve, out_reserve);
			ensure!(tokens_bought >= min_token, Error::<T>::InvalidValue);
			<TradingPairs<T>>::insert(
				&trading_pair,
				(input_reserve + tokens_sold, out_reserve - tokens_bought)
			);
			
			let key = (trading_pair.clone(), sender.clone());
			if let Some((balance_a, mut balance_b)) = Balance::<T>::get(&key) {
				balance_b = balance_b + tokens_bought;
				<Balance<T>>::insert(&key, (balance_a, balance_b));
			} else {
				<Balance<T>>::insert(&key, (0u128, tokens_bought));
			}

			Self::deposit_event(Event::SwapAtoBTokens(trading_pair, sender, tokens_sold, tokens_bought));
			Ok(())
		}

		/// Convert B token to A token
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())] 
		pub fn swap_btoa(origin: OriginFor<T>, trading_pair: Vec<u8>, tokens_sold: u128, min_token: u128, token_b_id: Vec<u8>, token_b_data: OmniverseTokenProtocol) -> DispatchResult{
			let sender = ensure_signed(origin)?;
			ensure!(tokens_sold > 0 && min_token > 0, Error::<T>::InvalidValue);
			// Transfer B token to MPC account
			T::OmniverseToken::send_transaction_external(token_b_id, &token_b_data).ok().ok_or(Error::<T>::OmniverseTransferFailed)?;

			let (input_reserve, out_reserve) = TradingPairs::<T>::get(&trading_pair).ok_or(Error::<T>::TradingPairNotExist)?;
			let tokens_bought = get_input_price(tokens_sold, out_reserve, input_reserve);
			ensure!(tokens_bought >= min_token, Error::<T>::InvalidValue);
			<TradingPairs<T>>::insert(
				&trading_pair,
				(input_reserve - tokens_bought, out_reserve + tokens_sold)
			);

			let key = (trading_pair.clone(), sender.clone());
			if let Some((mut balance_a, balance_b)) = Balance::<T>::get(&key) {
				balance_a = balance_a + tokens_bought;
				<Balance<T>>::insert(&key, (balance_a, balance_b));
			} else {
				<Balance<T>>::insert(&key, (tokens_bought, 0u128));
			}

			Self::deposit_event(Event::SwapBtoATokens(trading_pair, sender, tokens_sold, tokens_bought));
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())] 
		pub fn add_liquidity(origin: OriginFor<T>, trading_pair: Vec<u8>, amount_a_desired: u128, amount_b_desired: u128, amount_a_min: u128, amount_b_min: u128, token_a_id: Vec<u8>, token_a_data: OmniverseTokenProtocol, token_b_id: Vec<u8>, token_b_data: OmniverseTokenProtocol) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			ensure!(amount_a_desired > 0 && amount_b_desired > 0, Error::<T>::InvalidValue);
			let tranding_pair = TradingPairs::<T>::get(&trading_pair);
			let amount_a: u128;
			let amount_b: u128;
			if tranding_pair.is_some() {
				let (reserve_a, reserve_b) = TradingPairs::<T>::get(&trading_pair).ok_or(Error::<T>::TradingPairNotExist)?;
				let amount_b_optimal = quote(amount_a_desired, reserve_a, reserve_b);
				if amount_b_optimal <= amount_b_desired {
					ensure!(amount_b_optimal > 0 && amount_b_min > 0, Error::<T>::InsufficientBAmount);
					amount_a = amount_a_desired;
					amount_b = amount_b_optimal;
				} else {
					let amount_a_optimal = quote(amount_b_desired, reserve_b, reserve_a);
					ensure!(amount_a_optimal <= amount_a_desired, Error::<T>::ExceedDesiredAmount);
					ensure!(amount_a_optimal > 0 && amount_a_min > 0, Error::<T>::InsufficientAAmount);
					amount_a = amount_a_optimal;
					amount_b = amount_b_desired;
				}
				<TradingPairs<T>>::insert(
					&trading_pair,
					(reserve_a + amount_a, reserve_b + amount_b)
				);
			} else {
				amount_a = amount_a_desired;
				amount_b = amount_b_desired;
				<TradingPairs<T>>::insert(
					&trading_pair,
					(amount_a, amount_b)
				);
				<TotalLiquidity<T>>::insert(
					&trading_pair,
					0u128
				);
			}

			// transfer A token and B token to MPC address
			T::OmniverseToken::send_transaction_external(token_a_id, &token_a_data).ok().ok_or(Error::<T>::OmniverseTransferFailed)?;
			T::OmniverseToken::send_transaction_external(token_b_id, &token_b_data).ok().ok_or(Error::<T>::OmniverseTransferFailed)?;

			// mint
			let (balance_a, balance_b) = TradingPairs::<T>::get(&trading_pair).ok_or(Error::<T>::TradingPairNotExist)?;
			let mut total_supply = TotalLiquidity::<T>::get(&trading_pair).ok_or(Error::<T>::TradingPairNotExist)?;
			let liquidity: u128;
			if total_supply == 0 {
				liquidity = (amount_a * amount_b).pow(2u32).saturating_sub(1000);
				total_supply = liquidity;
			} else {
				// liquidity = Math.min(amount0.mul(_totalSupply) / _reserve0, amount1.mul(_totalSupply) / _reserve1);
				liquidity = (amount_a.saturating_mul(total_supply) / (balance_a - amount_a)).min(amount_b.saturating_mul(total_supply) / (balance_b - amount_b));
				total_supply += liquidity;
			}

			let key = (trading_pair.clone(), sender.clone());
			let balances = Liquidity::<T>::get(&key).unwrap_or(0) + liquidity;
			<Liquidity::<T>>::insert(&key, balances);
			<TotalLiquidity::<T>>::insert(&trading_pair, total_supply);

			Self::deposit_event(Event::AddLiquidity(trading_pair, sender, amount_a, amount_b));
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())] 
		pub fn remove_liquidity(origin: OriginFor<T>, trading_pair: Vec<u8>, liquidity: u128, amount_a_min: u128, amount_b_min: u128) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let key = (trading_pair.clone(), sender.clone());
			let balances = Liquidity::<T>::get(&key).unwrap_or(0);
			ensure!(balances >= liquidity, Error::<T>::InvalidValue);
			
			// burn
			let (reserve_a, reserve_b) = TradingPairs::<T>::get(&trading_pair).ok_or(Error::<T>::TradingPairNotExist)?;
			<Liquidity::<T>>::insert(&key, balances - liquidity);
			let total_supply = TotalLiquidity::<T>::get(&trading_pair).ok_or(Error::<T>::TradingPairNotExist)?;
			let amount_a = liquidity.saturating_mul(reserve_a) / total_supply;
			let amount_b = liquidity.saturating_mul(reserve_b) / total_supply;
			ensure!(amount_a >= amount_a_min && amount_b >= amount_b_min, Error::<T>::InsufficientAmount);

			<TotalLiquidity::<T>>::insert(&trading_pair, total_supply - liquidity);
			<TradingPairs::<T>>::insert(&trading_pair, (reserve_a - amount_a, reserve_b - amount_b));
			// MPC transfer A and B token to sender
			if let Some((mut balance_a, mut balance_b)) = Balance::<T>::get(&key) {
				balance_a = balance_a + amount_a;
				balance_b = balance_b + amount_b;
				<Balance<T>>::insert(&key, (balance_a, balance_b));
			} else {
				<Balance<T>>::insert(&key, (amount_a, amount_b));
			}
			
			Self::deposit_event(Event::RemoveLiquidity(trading_pair, sender, amount_a, amount_b));
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())] 
		pub fn transfer_a_token(origin: OriginFor<T>, trading_pair: Vec<u8>, to: T::AccountId, token_id: Vec<u8>, data: OmniverseTokenProtocol) -> DispatchResult {
			ensure_signed(origin)?;
			// TODO `to` need equal to `transfer_data.to`
			let key = (trading_pair.clone(), to);
			let op_data = TokenOpcode::decode(&mut data.data.as_slice()).unwrap();
			if op_data.op == TRANSFER {
				let transfer_data = TransferTokenOp::decode(&mut op_data.data.as_slice()).unwrap();
				if let Some((balance_a, balance_b)) = Balance::<T>::get(&key) {
					if transfer_data.amount <= balance_a {
						T::OmniverseToken::send_transaction_external(token_id, &data).ok().ok_or(Error::<T>::OmniverseTransferFailed)?;
						<Balance<T>>::insert(&key, (balance_a - transfer_data.amount, balance_b));
					}
				}
			}
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())] 
		pub fn transfer_b_token(origin: OriginFor<T>, trading_pair: Vec<u8>, to: T::AccountId, token_id: Vec<u8>, data: OmniverseTokenProtocol) -> DispatchResult {
			ensure_signed(origin)?;
			// TODO `to` need equal to `transfer_data.to`
			let key = (trading_pair.clone(), to);
			let op_data = TokenOpcode::decode(&mut data.data.as_slice()).unwrap();
			if op_data.op == TRANSFER {
				let transfer_data = TransferTokenOp::decode(&mut op_data.data.as_slice()).unwrap();
				if let Some((balance_a, balance_b)) = Balance::<T>::get(&key) {
					if transfer_data.amount <= balance_b {
						T::OmniverseToken::send_transaction_external(token_id, &data).ok().ok_or(Error::<T>::OmniverseTransferFailed)?;
						<Balance<T>>::insert(&key, (balance_a, balance_b - transfer_data.amount));
					}
				}
			}
			Ok(())
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
	pub fn quote(amount_a: u128, reserve_a: u128, reserve_b: u128) -> u128{
		amount_a * reserve_b / reserve_a
}
	// }
}
