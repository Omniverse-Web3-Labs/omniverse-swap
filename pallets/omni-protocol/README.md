# Omniverse Protocol Module

A module for managing omniverse accounts.

## Overview

The omniverse Protocol module will manage the underlying omniverse accounts, including:

* Omniverse Signature
* Account Nonce
* Transaction Verification

To use it in your runtime, you need to implement the Omniverse Protocol `omni-protocol::Config`.

The supported dispatchable functions are documented in the [`assets::Call`]().

### Terminology

* **Account nonce:** The account nonce is used to ensure that a user's transactions are executed sequentially.
* **Omniverse signature:** An omniverse transaction must be signed by a user with his private key. Currently, Secp256k1 is supported.
* **Transaction verification:** Before accepting an omniverse transaction, the signature, nonce and the hash of the transaction data will be verified.

### Goals

The omniverse protocol system in Substrate is designed to make the following possible:

* Manage omniverse accounts.
* Record transaction history.
* Verify omniverse transactions, add them to transaction history or reject them.

## Interface

### Dispatchable Functions

* `issue` - Issues the total supply of a new fungible asset to the account of the caller of the function.
* `transfer` - Transfers an `amount` of units of fungible asset `id` from the balance of
the function caller's account (`origin`) to a `target` account.
* `destroy` - Destroys the entire holding of a fungible asset `id` associated with the account
that called the function.

Please refer to the [`Call`](https://docs.rs/pallet-assets/latest/pallet_assets/enum.Call.html) enum and its associated variants for documentation on each function.

### Public Functions
<!-- Original author of descriptions: @gavofyork -->

* `balance` - Get the asset `id` balance of `who`.
* `total_supply` - Get the total supply of an asset `id`.

Please refer to the [`Pallet`](https://docs.rs/pallet-assets/latest/pallet_assets/pallet/struct.Pallet.html) struct for details on publicly available functions.

## Usage

The following example shows how to use the Assets module in your runtime by exposing public functions to:

* Issue a new fungible asset for a token distribution event (airdrop).
* Query the fungible asset holding balance of an account.
* Query the total supply of a fungible asset that has been issued.

### Prerequisites

Import the Assets module and types and derive your runtime's configuration traits from the Assets module trait.

### Simple Code Snippet

```rust
use pallet_assets as assets;
use sp_runtime::ArithmeticError;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config + assets::Config {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        pub fn issue_token_airdrop(origin: OriginFor<T>) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            const ACCOUNT_ALICE: u64 = 1;
            const ACCOUNT_BOB: u64 = 2;
            const COUNT_AIRDROP_RECIPIENTS: u64 = 2;
            const TOKENS_FIXED_SUPPLY: u64 = 100;

            ensure!(!COUNT_AIRDROP_RECIPIENTS.is_zero(), ArithmeticError::DivisionByZero);

            let asset_id = Self::next_asset_id();

            <NextAssetId<T>>::mutate(|asset_id| *asset_id += 1);
            <Balances<T>>::insert((asset_id, &ACCOUNT_ALICE), TOKENS_FIXED_SUPPLY / COUNT_AIRDROP_RECIPIENTS);
            <Balances<T>>::insert((asset_id, &ACCOUNT_BOB), TOKENS_FIXED_SUPPLY / COUNT_AIRDROP_RECIPIENTS);
            <TotalSupply<T>>::insert(asset_id, TOKENS_FIXED_SUPPLY);

            Self::deposit_event(Event::Issued(asset_id, sender, TOKENS_FIXED_SUPPLY));
            Ok(())
        }
    }
}
```

## Assumptions

Below are assumptions that must be held when using this module.  If any of
them are violated, the behavior of this module is undefined.

* The total count of assets should be less than
  `Config::AssetId::max_value()`.

## Related Modules

* [`System`](https://docs.rs/frame-system/latest/frame_system/)
* [`Support`](https://docs.rs/frame-support/latest/frame_support/)

License: Apache-2.0
