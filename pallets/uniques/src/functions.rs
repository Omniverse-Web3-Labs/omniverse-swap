// This file is part of Substrate.

// Copyright (C) 2017-2022 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Various pieces of common functionality.

use super::*;
use frame_support::{
	ensure,
	traits::{ExistenceRequirement, Get},
};
use pallet_omniverse_protocol::{
	traits::OmniverseAccounts,
	types::{Assets, OmniverseTransactionData, VerifyError, VerifyResult, BURN, MINT, TRANSFER},
};
use secp256k1::PublicKey;
use sp_core::Hasher;
use sp_runtime::traits::BlakeTwo256;
use sp_runtime::{DispatchError, DispatchResult};

impl<T: Config<I>, I: 'static> Pallet<T, I> {
	pub fn do_transfer(
		collection: T::CollectionId,
		item: T::ItemId,
		dest: T::AccountId,
		with_details: impl FnOnce(
			&CollectionDetailsFor<T, I>,
			&mut ItemDetailsFor<T, I>,
		) -> DispatchResult,
	) -> DispatchResult {
		let collection_details =
			Collection::<T, I>::get(&collection).ok_or(Error::<T, I>::UnknownCollection)?;
		ensure!(!collection_details.is_frozen, Error::<T, I>::Frozen);
		ensure!(!T::Locker::is_locked(collection, item), Error::<T, I>::Locked);

		let mut details =
			Item::<T, I>::get(&collection, &item).ok_or(Error::<T, I>::UnknownCollection)?;
		ensure!(!details.is_frozen, Error::<T, I>::Frozen);
		with_details(&collection_details, &mut details)?;

		Account::<T, I>::remove((&details.owner, &collection, &item));
		Account::<T, I>::insert((&dest, &collection, &item), ());
		let origin = details.owner;
		details.owner = dest;

		// The approved account has to be reset to None, because otherwise pre-approve attack would
		// be possible, where the owner can approve his second account before making the transaction
		// and then claiming the item back.
		details.approved = None;

		Item::<T, I>::insert(&collection, &item, &details);
		ItemPriceOf::<T, I>::remove(&collection, &item);

		Self::deposit_event(Event::Transferred {
			collection,
			item,
			from: origin,
			to: details.owner,
		});
		Ok(())
	}

	pub fn do_create_collection(
		collection: T::CollectionId,
		owner: T::AccountId,
		admin: T::AccountId,
		deposit: DepositBalanceOf<T, I>,
		free_holding: bool,
		event: Event<T, I>,
	) -> DispatchResult {
		ensure!(!Collection::<T, I>::contains_key(collection), Error::<T, I>::InUse);

		T::Currency::reserve(&owner, deposit)?;

		Collection::<T, I>::insert(
			collection,
			CollectionDetails {
				owner: owner.clone(),
				issuer: admin.clone(),
				admin: admin.clone(),
				freezer: admin,
				total_deposit: deposit,
				free_holding,
				items: 0,
				item_metadatas: 0,
				attributes: 0,
				is_frozen: false,
			},
		);

		CollectionAccount::<T, I>::insert(&owner, &collection, ());
		Self::deposit_event(event);
		Ok(())
	}

	pub fn do_destroy_collection(
		collection: T::CollectionId,
		witness: DestroyWitness,
		maybe_check_owner: Option<T::AccountId>,
	) -> Result<DestroyWitness, DispatchError> {
		Collection::<T, I>::try_mutate_exists(collection, |maybe_details| {
			let collection_details =
				maybe_details.take().ok_or(Error::<T, I>::UnknownCollection)?;
			if let Some(check_owner) = maybe_check_owner {
				ensure!(collection_details.owner == check_owner, Error::<T, I>::NoPermission);
			}
			ensure!(collection_details.items == witness.items, Error::<T, I>::BadWitness);
			ensure!(
				collection_details.item_metadatas == witness.item_metadatas,
				Error::<T, I>::BadWitness
			);
			ensure!(collection_details.attributes == witness.attributes, Error::<T, I>::BadWitness);

			for (item, details) in Item::<T, I>::drain_prefix(&collection) {
				Account::<T, I>::remove((&details.owner, &collection, &item));
			}
			#[allow(deprecated)]
			ItemMetadataOf::<T, I>::remove_prefix(&collection, None);
			#[allow(deprecated)]
			ItemPriceOf::<T, I>::remove_prefix(&collection, None);
			CollectionMetadataOf::<T, I>::remove(&collection);
			#[allow(deprecated)]
			Attribute::<T, I>::remove_prefix((&collection,), None);
			CollectionAccount::<T, I>::remove(&collection_details.owner, &collection);
			T::Currency::unreserve(&collection_details.owner, collection_details.total_deposit);
			CollectionMaxSupply::<T, I>::remove(&collection);

			Self::deposit_event(Event::Destroyed { collection });

			Ok(DestroyWitness {
				items: collection_details.items,
				item_metadatas: collection_details.item_metadatas,
				attributes: collection_details.attributes,
			})
		})
	}

	pub fn do_mint(
		collection: T::CollectionId,
		item: T::ItemId,
		owner: T::AccountId,
		with_details: impl FnOnce(&CollectionDetailsFor<T, I>) -> DispatchResult,
	) -> DispatchResult {
		ensure!(!Item::<T, I>::contains_key(collection, item), Error::<T, I>::AlreadyExists);

		Collection::<T, I>::try_mutate(
			&collection,
			|maybe_collection_details| -> DispatchResult {
				let collection_details =
					maybe_collection_details.as_mut().ok_or(Error::<T, I>::UnknownCollection)?;

				with_details(collection_details)?;

				if let Ok(max_supply) = CollectionMaxSupply::<T, I>::try_get(&collection) {
					ensure!(collection_details.items < max_supply, Error::<T, I>::MaxSupplyReached);
				}

				let items =
					collection_details.items.checked_add(1).ok_or(ArithmeticError::Overflow)?;
				collection_details.items = items;

				let deposit = match collection_details.free_holding {
					true => Zero::zero(),
					false => T::ItemDeposit::get(),
				};
				T::Currency::reserve(&collection_details.owner, deposit)?;
				collection_details.total_deposit += deposit;

				let owner = owner.clone();
				Account::<T, I>::insert((&owner, &collection, &item), ());
				let details = ItemDetails { owner, approved: None, is_frozen: false, deposit };
				Item::<T, I>::insert(&collection, &item, details);
				Ok(())
			},
		)?;

		Self::deposit_event(Event::Issued { collection, item, owner });
		Ok(())
	}

	pub fn do_burn(
		collection: T::CollectionId,
		item: T::ItemId,
		with_details: impl FnOnce(&CollectionDetailsFor<T, I>, &ItemDetailsFor<T, I>) -> DispatchResult,
	) -> DispatchResult {
		let owner = Collection::<T, I>::try_mutate(
			&collection,
			|maybe_collection_details| -> Result<T::AccountId, DispatchError> {
				let collection_details =
					maybe_collection_details.as_mut().ok_or(Error::<T, I>::UnknownCollection)?;
				let details = Item::<T, I>::get(&collection, &item)
					.ok_or(Error::<T, I>::UnknownCollection)?;
				with_details(collection_details, &details)?;

				// Return the deposit.
				T::Currency::unreserve(&collection_details.owner, details.deposit);
				collection_details.total_deposit.saturating_reduce(details.deposit);
				collection_details.items.saturating_dec();
				Ok(details.owner)
			},
		)?;

		Item::<T, I>::remove(&collection, &item);
		Account::<T, I>::remove((&owner, &collection, &item));
		ItemPriceOf::<T, I>::remove(&collection, &item);

		Self::deposit_event(Event::Burned { collection, item, owner });
		Ok(())
	}

	pub fn do_set_price(
		collection: T::CollectionId,
		item: T::ItemId,
		sender: T::AccountId,
		price: Option<ItemPrice<T, I>>,
		whitelisted_buyer: Option<T::AccountId>,
	) -> DispatchResult {
		let details = Item::<T, I>::get(&collection, &item).ok_or(Error::<T, I>::UnknownItem)?;
		ensure!(details.owner == sender, Error::<T, I>::NoPermission);

		if let Some(ref price) = price {
			ItemPriceOf::<T, I>::insert(&collection, &item, (price, whitelisted_buyer.clone()));
			Self::deposit_event(Event::ItemPriceSet {
				collection,
				item,
				price: *price,
				whitelisted_buyer,
			});
		} else {
			ItemPriceOf::<T, I>::remove(&collection, &item);
			Self::deposit_event(Event::ItemPriceRemoved { collection, item });
		}

		Ok(())
	}

	pub fn do_buy_item(
		collection: T::CollectionId,
		item: T::ItemId,
		buyer: T::AccountId,
		bid_price: ItemPrice<T, I>,
	) -> DispatchResult {
		let details = Item::<T, I>::get(&collection, &item).ok_or(Error::<T, I>::UnknownItem)?;
		ensure!(details.owner != buyer, Error::<T, I>::NoPermission);

		let price_info =
			ItemPriceOf::<T, I>::get(&collection, &item).ok_or(Error::<T, I>::NotForSale)?;

		ensure!(bid_price >= price_info.0, Error::<T, I>::BidTooLow);

		if let Some(only_buyer) = price_info.1 {
			ensure!(only_buyer == buyer, Error::<T, I>::NoPermission);
		}

		T::Currency::transfer(
			&buyer,
			&details.owner,
			price_info.0,
			ExistenceRequirement::KeepAlive,
		)?;

		let old_owner = details.owner.clone();

		Self::do_transfer(collection, item, buyer.clone(), |_, _| Ok(()))?;

		Self::deposit_event(Event::ItemBought {
			collection,
			item,
			price: price_info.0,
			seller: old_owner,
			buyer,
		});

		Ok(())
	}

	pub fn to_account(public_key: &[u8; 64]) -> Result<T::AccountId, Error<T, I>> {
		let mut pk_full: [u8; 65] = [0; 65];
		pk_full[1..65].copy_from_slice(public_key);
		pk_full[0] = 4;
		let public_key = PublicKey::from_slice(&pk_full[..])
			.map_err(|_| Error::<T, I>::SerializePublicKeyFailed)?;
		let public_key_compressed = public_key.serialize();
		let hash = BlakeTwo256::hash(&public_key_compressed);
		Ok(T::AccountId::decode(&mut &hash[..]).unwrap())
	}

	pub(super) fn omniverse_transfer(
		omniverse_token: OmniverseToken<T::AccountId>,
		from: [u8; 64],
		to: [u8; 64],
		quantity: u128,
	) -> Result<(), DispatchError> {
		// let from_asset = Tokens::<T, I>::get(&omniverse_token.token_id, &from);
		// let assets = Tokens::<T, I>::get(&omniverse_token.token_id, &from);
		if let Some(mut assets) = Tokens::<T, I>::get(&omniverse_token.token_id, &from) {
			if assets.contains(&quantity) {
				assets.retain(|&x| x != quantity);
			}
			Tokens::<T, I>::insert(&omniverse_token.token_id, &from, assets);
			let mut dest_assets =
				Tokens::<T, I>::get(&omniverse_token.token_id, &to).unwrap_or(Vec::new());
			dest_assets.push(quantity);
			Tokens::<T, I>::insert(&omniverse_token.token_id, &to, dest_assets);
		} else {
			return Err(Error::<T, I>::UnknownCollection.into());
		}
		Ok(())
	}

	pub(super) fn omniverse_mint(
		omniverse_token: OmniverseToken<T::AccountId>,
		to: [u8; 64],
		quantity: u128,
	) -> Result<(), DispatchError> {
		// let from_asset = Tokens::<T, I>::get(&omniverse_token.token_id, &from);
		// let assets = Tokens::<T, I>::get(&omniverse_token.token_id, &from);
		let mut assets = Tokens::<T, I>::get(&omniverse_token.token_id, &to).unwrap_or(Vec::new());
		if assets.contains(&quantity) {
			return Err(Error::<T, I>::AlreadyExists.into());
		}
		assets.push(quantity);
		Tokens::<T, I>::insert(&omniverse_token.token_id, &to, assets);
		Ok(())
	}

	pub(super) fn omniverse_burn(
		omniverse_token: OmniverseToken<T::AccountId>,
		account: [u8; 64],
		quantity: u128,
	) -> Result<(), DispatchError> {
		// let from_asset = Tokens::<T, I>::get(&omniverse_token.token_id, &from);
		// let assets = Tokens::<T, I>::get(&omniverse_token.token_id, &from);
		let mut assets =
			Tokens::<T, I>::get(&omniverse_token.token_id, &account).unwrap_or(Vec::new());
		if assets.contains(&quantity) {
			assets.retain(|&x| x != quantity);
		} else {
			return Err(Error::<T, I>::NotExist.into());
		}
		Tokens::<T, I>::insert(&omniverse_token.token_id, &account, assets);
		Ok(())
	}

	pub fn send_transaction_external(
		token_id: Vec<u8>,
		data: &OmniverseTransactionData,
	) -> Result<FactoryResult, DispatchError> {
		// Check if the token exists.
		let token = TokensInfo::<T, I>::get(&token_id).ok_or(Error::<T, I>::UnknownCollection)?;

		Self::handle_transaction(token, data)?;

		Ok(FactoryResult::Success)
	}

	pub(super) fn handle_transaction(
		omniverse_token: OmniverseToken<T::AccountId>,
		data: &OmniverseTransactionData,
	) -> Result<FactoryResult, DispatchError> {
		// Check if the tx destination is correct
		ensure!(
			omniverse_token.is_member(&(data.chain_id, data.initiator_address.clone()))
				|| data.initiator_address == omniverse_token.token_id,
			Error::<T, I>::WrongDestination
		);

		// Check if the sender is honest
		ensure!(!T::OmniverseProtocol::is_malicious(data.from), Error::<T, I>::UserIsMalicious);

		// Verify the signature
		let ret = T::OmniverseProtocol::verify_transaction(
			&PALLET_NAME.to_vec(),
			&omniverse_token.token_id,
			&data,
			false,
		);
		let ret = match ret {
			Err(_) => T::OmniverseProtocol::verify_transaction(
				&PALLET_NAME.to_vec(),
				&omniverse_token.token_id,
				&data,
				true,
			),
			_ => ret,
		};
		let source = Self::to_account(&data.from)?;

		match ret {
			Ok(VerifyResult::Malicious) => return Ok(FactoryResult::ProtocolMalicious),
			Ok(VerifyResult::Duplicated) => return Ok(FactoryResult::ProtocolDuplicated),
			Err(VerifyError::SignatureError) => {
				return Err(Error::<T, I>::ProtocolSignatureError.into())
			},
			Err(VerifyError::SignerNotCaller) => {
				return Err(Error::<T, I>::ProtocolSignerNotCaller.into())
			},
			Err(VerifyError::NonceError) => return Err(Error::<T, I>::ProtocolNonceError.into()),
			Ok(VerifyResult::Success) => {
				// Verify balance
				{
					let id = TokenId2CollectionId::<T, I>::get(&omniverse_token.token_id)
						.ok_or(Error::<T, I>::UnknownCollection)?;
					let assets = Assets::decode(&mut data.payload.as_slice())
						.map_err(|_| Error::<T, I>::DecodePayloadFailed)?;
					let item = T::ItemId::try_from(assets.quantity)
						.unwrap_or(<T as Config<I>>::ItemId::default());
					let collection_details =
						Collection::<T, I>::get(&id).ok_or(Error::<T, I>::UnknownCollection)?;
					if assets.op == TRANSFER {
						let dest_pk: [u8; 64] = assets
							.ex_data
							.try_into()
							.map_err(|_| Error::<T, I>::SerializePublicKeyFailed)?;
						Self::to_account(&dest_pk)?;
						ensure!(!collection_details.is_frozen, Error::<T, I>::Frozen);
						ensure!(!T::Locker::is_locked(id, item), Error::<T, I>::Locked);

						let mut details = Item::<T, I>::get(&id, &item)
							.ok_or(Error::<T, I>::UnknownCollection)?;
						ensure!(!details.is_frozen, Error::<T, I>::Frozen);
						if details.owner != source && collection_details.admin != source {
							let approved = details.approved.take().map_or(false, |i| i == source);
							ensure!(approved, Error::<T, I>::NoPermission);
						}
					} else if assets.op == MINT {
						let dest_pk: [u8; 64] = assets
							.ex_data
							.try_into()
							.map_err(|_| Error::<T, I>::SerializePublicKeyFailed)?;
						Self::to_account(&dest_pk)?;
						ensure!(
							!Item::<T, I>::contains_key(id, item),
							Error::<T, I>::AlreadyExists
						);
						ensure!(collection_details.issuer == source, Error::<T, I>::NoPermission);

						if let Ok(max_supply) = CollectionMaxSupply::<T, I>::try_get(&id) {
							ensure!(
								collection_details.items < max_supply,
								Error::<T, I>::MaxSupplyReached
							);
						}
						collection_details.items.checked_add(1).ok_or(ArithmeticError::Overflow)?;
					} else if assets.op == BURN {
						let details = Item::<T, I>::get(&id, &item)
							.ok_or(Error::<T, I>::UnknownCollection)?;
						let is_permitted = details.owner == source;
						ensure!(is_permitted, Error::<T, I>::NoPermission);
					} else {
						return Err(Error::<T, I>::UnknownProtocolType.into());
					}
				}
				let (delayed_executing_index, delayed_index) = DelayedIndex::<T, I>::get();
				DelayedTransactions::<T, I>::insert(
					delayed_index,
					DelayedTx::new(data.from, omniverse_token.token_id.clone(), data.nonce),
				);
				DelayedIndex::<T, I>::set((delayed_executing_index, delayed_index + 1));
				Self::deposit_event(Event::TransactionSent {
					pk: data.from,
					token_id: omniverse_token.token_id,
					nonce: data.nonce,
				});
			},
		}

		Ok(FactoryResult::Success)
	}

	pub(super) fn execute_transaction(
		token_id: &Vec<u8>,
		data: &OmniverseTransactionData,
	) -> Result<(), DispatchError> {
		let omniverse_token =
			TokensInfo::<T, I>::get(&token_id).ok_or(Error::<T, I>::UnknownCollection)?;

		// Execute
		// let op_data = TokenOpcode::decode(&mut data.data.as_slice()).unwrap();
		// let transfer_data = TransferTokenOp::decode(&mut data.op_data.as_slice()).unwrap();
		let assets = Assets::decode(&mut data.payload.as_slice())
			.map_err(|_| Error::<T, I>::DecodePayloadFailed)?;
		// Convert public key to account id
		let origin = Self::to_account(&data.from)?;
		let item_id =
			T::ItemId::try_from(assets.quantity).unwrap_or(<T as Config<I>>::ItemId::default());
		let id =
			TokenId2CollectionId::<T, I>::get(token_id).ok_or(Error::<T, I>::UnknownCollection)?;

		if assets.op == TRANSFER {
			let dest_pk: [u8; 64] =
				assets.ex_data.try_into().map_err(|_| Error::<T, I>::SerializePublicKeyFailed)?;
			let dest = Self::to_account(&dest_pk)?;
			Self::do_transfer(id, item_id, dest, |collection_details, details| {
				if details.owner != origin && collection_details.admin != origin {
					let approved = details.approved.take().map_or(false, |i| i == origin);
					ensure!(approved, Error::<T, I>::NoPermission);
				}
				Self::omniverse_transfer(omniverse_token, data.from, dest_pk, assets.quantity)?;
				Ok(())
			})?;
		} else if assets.op == MINT {
			let dest_pk: [u8; 64] =
				assets.ex_data.try_into().map_err(|_| Error::<T, I>::SerializePublicKeyFailed)?;
			let dest = Self::to_account(&dest_pk)?;
			Self::do_mint(id, item_id, dest, |collection_details| {
				ensure!(collection_details.issuer == origin, Error::<T, I>::NoPermission);
				Ok(())
			})?;
			Self::omniverse_mint(omniverse_token, dest_pk, assets.quantity)?;
		} else if assets.op == BURN {
			// let check_owner = Some(origin.clone());
			Self::do_burn(id, item_id, |_, details| {
				let is_permitted = details.owner == origin;
				ensure!(is_permitted, Error::<T, I>::NoPermission);
				// ensure!(
				// 	check_owner.map_or(true, |o| o == details.owner),
				// 	Error::<T, I>::WrongOwner
				// );
				Ok(())
			})?;
			Self::omniverse_burn(omniverse_token, data.from, assets.quantity)?;
		}
		Ok(())
	}
}
