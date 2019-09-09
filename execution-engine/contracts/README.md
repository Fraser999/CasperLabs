# Contracts

## Overview

This directory contains many individual Rust crates, the vast majority of which compile to Wasm binaries for use as
contracts.

There is a top-level phony library target which is only used as a mechanism to allow these Wasm targets to be specified
as build dependencies of other parts of the system.

This top-level crate also includes features corresponding to the subdirectories containing contracts.  This allows
subsets to be built or specified as dependencies.  For example, the `casperlabs-engine-core` crate only depends upon the
system contracts via the following dependency in "execution-engine/engine-core/Cargo.toml":

```toml
contracts = { path = "../contracts", package = "casperlabs-contracts", default-features = false, features = ["system"] }
```

## Building

To build all contracts:

```
cd execution-engine/contracts/
cargo build --release
```

To build a subset of contracts (e.g. all those in "contracts/system"):

```
cd execution-engine/contracts/
cargo build --release --no-default-features --features=system
```

To build a single contract (e.g. "bonding" in "contracts/client"):

```
cd execution-engine/contracts/
cargo build --release --package=bonding
```

When built, the compiled Wasm files will be found in "execution-engine/target/wasm32-unknown-unknown/release".


If building all contracts, or a subset using the feature flag, you can specify a different output directory by setting
the environment variable `CL_WASM_DIR` to the desired path.  This directory and any required parents will be created if
they don't already exist.  Existing Wasm files will be overwritten by newer ones produced by the build process.

For example, to build all contracts to "execution-engine/resources":

```
cd execution-engine/contracts/
CL_WASM_DIR=../resources cargo build --release
```

The output directory cannot be changed when building a single contract.
