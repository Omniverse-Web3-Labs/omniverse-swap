# Omniverse DLT of Substrate

The Omniverse DLT is a new application-level token protocol built over multiple existing L1 public chains, enabling asset-related operations such as transfers and receptions running over different consensus spaces synchronously and equivalently.

This repository is the implementation of Omniverse Protocol in form of pallets of Substrate.

## Getting Started

1.Learn the [Components](#components) of Omniverse DLT.  
2.[Develop](#development) your own Omniverse Tokens.  
3.[Run Omniverse DLT node](#run-node).

## Components

The project is built based on [Node Template](https://github.com/substrate-developer-hub/substrate-node-template), you can go there for more details.

### Pallets
- [Assets](./pallets/assets/README.md): A simple, secure module for dealing with fungible assets.
- [Uniques](./pallets/uniques/README.md): A simple, secure module for dealing with non-fungible assets.
- [Omniverse Protocol](./pallets/omni-protocol/README.md): A module for managing Omniverse accounts.

## Development

### Create a pallet

If you want to create a new Omniverse dApp, you need to create a pallet first, you can refer to the document of [Substrate](https://docs.substrate.io/tutorials/work-with-pallets/add-a-pallet/).

### Implement the method `send_transaction`

The method `send_transaction` is used to commit an Omniverset transaction to the chain you are building, the declaration is like this:

```
pub fn send_transaction(
			origin: OriginFor<T>,
			data: OmniverseTransactionData,
		) -> DispatchResult;
```

- origin: The account who signs the transaction  
- data: The data for the Omniverse transaction

You will see that there is also a paremeter `token_id` in Assets module, it is not needed by all bussiness.

### Write bussiness code

Feel free to write your bussiness, you can also refer to the examples we provide: [`Assets`](./pallets/assets/) and [`Uniques`](./pallets/uniques/).

If the your code needs to interact with `Omniverse Protocol` module, you can refer to the document of [`Omniverse Protocol`](./pallets/omni-protocol/README.md).

## Run node

Building and running a node follows the tutorial of [Substrate Node](https://github.com/substrate-developer-hub/substrate-node-template/blob/main/README.md).