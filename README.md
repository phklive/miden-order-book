![Miden CLOB logo](assets/logo.jpeg)

This CLI tool provides an interface for interacting with a central limit order book implemented on the Miden rollup.

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

Make sure you have Rust and Cargo installed on your system before proceeding with the installation. If you don't have Rust installed, you can get it from [https://rustup.rs/](https://rustup.rs/).

## Usage

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

## Commands

The Miden Order Book CLI currently supports the following commands:

- `sync`: Synchronize the local state with the Miden rollup.
- `setup`: Deploy 50 swap notes to the Miden rollup.

For more details on each command, you can use the `--help` flag:

```
miden-order-book --help
```

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
