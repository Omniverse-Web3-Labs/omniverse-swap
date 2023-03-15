# Omniverse Assets Module

A simple, secure module for dealing with Omniverse fungible assets, based on pallet_asset.

## Overview

The Assets module provides functionality for asset management of Omniverse fungible asset classes, including:

* Fungible asset Create
* Fungible asset Mint
* Fungible asset Transfer
* Fungible asset Burn

### Terminology

* **Omniverse Account:** The o-account is be expressed as a public key created by the elliptic curve secp256k1.
* **Account nonce:** The account nonce is used to ensure that a user's transactions are executed sequentially.
* **Omniverse signature:** An omniverse transaction must be signed by a user with his private key. Currently, Secp256k1 is supported.
* **Transaction verification:** Before accepting an omniverse transaction, the signature, nonce and the hash of the transaction data will be verified.
* **Asset Create:** The creation of a new asset.
* **Asset Mint:** Token owner mint amount ominverse token to one o-account.
* **Asset transfer:** The action of transferring assets from one o-account to another o-account.
* **Asset Burn:** The process of an account removing its entire holding of an asset.
* **Fungible asset:** An asset whose units are interchangeable.

### Goals

The assets system in Substrate is designed to make the following possible:

* Mint a unique Omniverse asset to one o-account.
* Move Omniverse assets between o-accounts.
* Remove an o-account's balance of an Omniverse asset when requested by that o-account's owner and update
  the omniverse asset's total supply.

## Interface

### Dispatchable Functions

* `create_token` - Create a new fungible asset.
* `send_transaction` - Send an omniverse transaction

### Public Functions

* `tokens` - Get the Omniverse asset `token_id` balance of `who`.
* `tokens_info` - Get the owner and members of an Omniverse asset `token_id`.

## Usage

The following example shows how to use the Omniverse Assets module in your runtime by exposing public functions to:

* Create new Omniverse token.
* Send an omniverse transaction
* Query the omniverse fungible asset holding balance of an account.
* Query the owner and members of an Omniverse asset.

### Simple Code Snippet

#### Create new Omniverse token

`token_id` and `asset_id` can be mapped to each other. `asset_id` is self-increase, users can't specify by themselves.

```rust
#[pallet::weight(0)]
pub fn create_token(
  origin: OriginFor<T>,
  owner_pk: [u8; 64],
  token_id: Vec<u8>,
  members: Option<Vec<(u32, Vec<u8>)>>,
) -> DispatchResult {
  ...
  // Check if the token exists
  ensure!(!TokensInfo::<T, I>::contains_key(&token_id), Error::<T, I>::InUse);
  // Update storage.
  TokensInfo::<T, I>::insert(
    &token_id,
    OmniverseToken::new(owner.clone(), owner_pk, token_id.clone(), members.clone()),
  );
  ...
  let mut id = CurrentAssetId::<T, I>::get(&token_id).unwrap_or_default();
  while Asset::<T, I>::contains_key(id) {
    id.saturating_inc();
  }

  AssetId2TokenId::<T, I>::insert(&id, token_id.clone());
  TokenId2AssetId::<T, I>::insert(&token_id, id.clone());

  Asset::<T, I>::insert(
    id,
    AssetDetails {
      owner: owner.clone(),
      issuer: admin.clone(),
      admin: admin.clone(),
      freezer: admin.clone(),
      supply: Zero::zero(),
      deposit,
      min_balance: Zero::zero(),
      is_sufficient: false,
      accounts: 0,
      sufficients: 0,
      approvals: 0,
      is_frozen: false,
    },
  );
  Self::deposit_event(Event::Created { asset_id: id, creator: owner, owner: admin });
  Ok(())
}
```

#### Send an omniverse transaction

`send_transaction` will verify the legitimacy of the transaction, including Omniverse signature and pallet-assets related verification. After successful verification, it will be inserted into the delayed queue and then wait to be exected.

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
