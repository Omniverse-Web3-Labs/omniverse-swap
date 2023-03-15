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

## Run node

Building and running a node follows the tutorial of [Substrate Node](https://github.com/substrate-developer-hub/substrate-node-template/blob/main/README.md).