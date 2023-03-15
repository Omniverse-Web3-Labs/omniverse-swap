# Omniverse Protocol Module

A module for managing omniverse accounts.

## Overview

The omniverse protocol module will manage the underlying omniverse accounts, including:

* Omniverse Signature
* Account Nonce
* Transaction Verification

To use it in your runtime, you need to implement the Omniverse Protocol `omni-protocol::Config`.

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

The pallet supply the underlying functions for omniverse applications, so it has **no dispatchable functions**.

### Public Functions

* `get_transaction_count` - Get the transaction count of a user.  
* `get_transaction_data` - Get the transaction data of a user at a nonce.  
* `is_malicious` - Get the maliciousness of a user.  
* `get_cooling_down_time` - Get the time within which only one transaction can be sent.  
* `get_chain_id` - Get the omniverse chain id.  
* `verify_transaction` - Verify an omniverse transaction, and return the result.


## Usage

The following example shows how to use the Omniverse Protocol module in your runtime by exposing public functions to:

* Verify an omniverse transaction.
* Query the maliciousness of a user.

### Prerequisites

Import the Omniverse Protocol module and types and add configuration field with Omniverse Protocol traits into the `Config`.

### Simple Code Snippet

```rust
pub static PALLET_NAME: [u8; 6] = [0x61, 0x73, 0x73, 0x65, 0x74, 0x73];

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use pallet_omniverse_protocol::{traits::OmniverseAccounts, OmniverseTransactionData};

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
      type OmniverseProtocol: OmniverseAccounts;
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        pub(super) fn handle_transaction(
          token_id: Vec<u8>,
          data: &OmniverseTransactionData,
        ) -> Result<(), DispatchError> {
          // Check if the sender is honest
          ensure!(!T::OmniverseProtocol::is_malicious(data.from), Error::<T>::UserIsMalicious);

          // Verify the signature
          let ret = T::OmniverseProtocol::verify_transaction(
            &PALLET_NAME.to_vec(),
            &token_id,
            &data,
          );

          Ok(())
        }
    }
}
```
