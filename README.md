![Miden CLOB logo](assets/logo.jpeg)

This CLI tool provides an interface for interacting with a central limit order book implemented on the Miden rollup.

## Prerequisites

Before you begin, ensure you have the following prerequisites:

1. **Miden Node**: You need to run a Miden node. For installation and setup instructions, refer to the [Miden Node GitHub repository](https://github.com/0xPolygonMiden/miden-node).

2. **Rust and Cargo**: This project is built with Rust. If you don't have Rust and Cargo installed, you can get them from the [Rust website](https://www.rust-lang.org/tools/install). Follow the installation instructions for your operating system.

## Installation

To install the Miden Order Book CLI, follow these steps:

1. Clone the repository:
   ```
   git clone https://github.com/phklive/miden-order-book.git
   cd miden-order-book
   ```

2. Install the CLI using Cargo:
   ```
   cargo install --path .
   ```

This will compile the project and install the `miden-order-book` binary in your Cargo binary directory.

## Usage

### Initializing the Environment

To set up the order book environment and remove any existing database, use the `init` command:

```
miden-order-book init
```

This command will:
1. Check for an existing `store.sqlite3` file in the current directory
2. If the file exists, it will be deleted
3. Prepare the environment for a fresh start

It's recommended to run this command when you want to reset your local state or start with a clean slate.

### Syncing the Rollup State

To synchronize the state of the rollup and update your local state, use the `sync` command:

```
miden-order-book sync
```

This command will:
1. Connect to the Miden rollup
2. Fetch the latest state
3. Update your local state to reflect the current rollup state

It's recommended to run this command before performing any operations to ensure you're working with the most up-to-date information.

### Deploying the CLOB

To deploy the central limit order book on Miden, use the `setup` command:

```
miden-order-book setup
```

This command will:
1. Create 50 swap notes
2. Each note will contain `ASSETA` and request `ASSETB`
3. Deploy these notes to the Miden rollup

This setup process simulates creating multiple limit orders in the order book.

### Executing an order

To execute an order using the limit order book on Miden, use the `order` command followed by the `<type>` of order:

```
miden-order-book order <type> <amount_1> <faucet_id_1> <amount_2> <faucet_id_2>
```

This command will:
1. Query all relevant notes that can fullfill the order request
2. Execute the order and transition local state
3. Submit updated state to the rollup

## Commands

The Miden Order Book CLI currently supports the following commands:

| Command | Description | Usage |
|---------|-------------|-------|
| `init`  | Initialize or reset the order book environment | `miden-order-book init` |
| `sync`  | Synchronize the local state with the Miden rollup | `miden-order-book sync` |
| `setup` | Deploy 50 swap notes to the Miden rollup | `miden-order-book setup` |
| `order` | Execute a `buy` or `sell` order | `miden-order-book order <type>` |

For more details on each command, you can use the `--help` flag:

```
miden-order-book --help
miden-order-book <command> --help
```

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
