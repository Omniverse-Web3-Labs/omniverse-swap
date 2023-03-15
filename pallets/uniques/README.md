# Omniverse Uniques Module

A simple, secure module for dealing with omniverse non-fungible assets, based on pallet_uniques.

## Overview

The Uniques module provides functionality for asset management of Omniverse non-fungible asset classes, including:

* Non-fungible asset Create
* Non-fungible asset Mint
* Non-fungible asset Transfer
* Non-fungible asset Burn

### Terminology

* **Omniverse Account:** The o-account is be expressed as a public key created by the elliptic curve secp256k1.
* **Account nonce:** The account nonce is used to ensure that a user's transactions are executed sequentially.
* **Omniverse signature:** An omniverse transaction must be signed by a user with his private key. Currently, Secp256k1 is supported.
* **Transaction verification:** Before accepting an omniverse transaction, the signature, nonce and the hash of the transaction data will be verified.
* **Non-fungible asset Create:** The creation of a new non-fungible asset.
* **Non-fungible asset Mint:** Token owner mint amount ominverse token to one o-account.
* **Non-fungible asset transfer:** The action of transferring non-fungible assets from one o-account to another o-account.
* **Non-fungible asset Burn:** The process of an account removing its entire holding of an non-fungible asset.
* **Non-fungible asset :** An asset for which each unit has unique characteristics.

### Goals

The Omniverse Uniques pallet in Substrate is designed to make the following possible:

* Allow o-accounts to permissionlessly create asset classes (collections of asset instances).
* Allow a named (permissioned) o-account to mint and burn Omniverse unique assets within a class.
* Move asset instances between o-accounts permissionlessly.

## Interface

### Dispatchable Functions
* `create_token` - Create a new non-fungible asset.
* `send_transaction` - Send an omniverse transaction

### Public Functions
* `tokens` - Get the Omniverse asset `token_id` balance of `who`.
* `tokens_info` - Get the owner and members of an Omniverse asset `token_id`.

### Metadata (permissioned) dispatchables
* `set_metadata`: Set general metadata of an asset instance.
* `clear_metadata`: Remove general metadata of an asset instance.
* `set_class_metadata`: Set general metadata of an asset class.
* `clear_class_metadata`: Remove general metadata of an asset class.

## Usage

The following example shows how to use the Omniverse Uniques module in your runtime by exposing public functions to:

* Create new Omniverse token.
* Send an omniverse transaction
* Query the omniverse non-fungible asset holding balance of an account.
* Query the owner and members of an Omniverse non-fungible asset.

### Simple Code Snippet

#### Create new Omniverse token

`token_id` and `collection_id` can be mapped to each other. `collection_id` is self-increase, users can't specify by themselves.

```rust
#[pallet::weight(0)]
pub fn create_token(
  origin: OriginFor<T>,
  owner_pk: [u8; 64],
  token_id: Vec<u8>,
  members: Option<Vec<(u32, Vec<u8>)>>,
) -> DispatchResult {
  ...
  // Update storage.
  TokensInfo::<T, I>::insert(
    &token_id,
    OmniverseToken::new(owner.clone(), owner_pk, token_id.clone(), members.clone()),
  );
  
  ...
  let mut id = CurrentCollectionId::<T, I>::get().unwrap_or_default();
  while Collection::<T, I>::contains_key(id) {
    id.saturating_inc();
  }

  CollectionId2TokenId::<T, I>::insert(&id, token_id.clone());
  TokenId2CollectionId::<T, I>::insert(&token_id, id.clone());

  Self::do_create_collection(
    id,
    owner.clone(),
    owner.clone(),
    T::CollectionDeposit::get(),
    false,
    Event::Created { collection: id, creator: owner.clone(), owner },
  )
}
```

#### Send an omniverse transaction

`send_transaction` will verify the legitimacy of the transaction, including Omniverse signature and pallet-uniques related verification. After successful verification, it will be inserted into the delayed queue and then wait to be exected.

```rust
#[pallet::weight(0)]
pub fn send_transaction(
  origin: OriginFor<T>,
  token_id: Vec<u8>,
  data: OmniverseTransactionData,
) -> DispatchResult {
  ensure_signed(origin)?;

  Self::send_transaction_external(token_id, &data)?;

  Ok(())
}

fn send_transaction_external(
  token_id: Vec<u8>,
  data: &OmniverseTransactionData,
) -> Result<FactoryResult, DispatchError> {
  ...
  Self::handle_transaction(token, data)?;
  ...
}

pub(super) fn handle_transaction(
		omniverse_token: OmniverseToken<T::AccountId>,
		data: &OmniverseTransactionData,
	) -> Result<FactoryResult, DispatchError> {
  ...
  // Verify the signature
  let ret = T::OmniverseProtocol::verify_transaction(
    &PALLET_NAME.to_vec(),
    &omniverse_token.token_id,
    &data,
  );
  ...
  match ret {
    ...
    Ok(VerifyResult::Success) => {
      // Verify balance
      {
        ...
        if fungible.op == TRANSFER {
          ...
        } else if fungible.op == MINT {
          ...
        } else if fungible.op == BURN {
          ...
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
    },
  }
  ...
}
```

License: Apache-2.0
