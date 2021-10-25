# stackmate-core

A Rust-C FFI library exposing composite functionality from bdk & rust-bitcoin; to create cross-platform descriptor wallets.

## overview

The entire ffi uses a string interface; defined in `src/lib.rs`

1. Inputs are converted into native rust types as the first sanitization step. 

2. Native types are then used in pure rust modules.

3. All native structs being returned `impl` a `c_stringify` method which converts the native struct into stringified JSON outputted as a CString.

## build

`resources/build.sh` currently only supports android builds. 

Binaries are zipped into `resources/builds.zip`.

## docs

More documentation can be found in `resources/docs`