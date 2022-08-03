#!/bin/bash
cd ..
cargo clippy

# export CARGO_INCREMENTAL=0
# export RUSTFLAGS="-Zprofile -Zinstrument-coverage -Ccodegen-units=1 -Copt-level=0 -Clink-dead-code -Coverflow-checks=off -Zpanic_abort_tests -Cpanic=abort"
# export RUSTDOCFLAGS="-Cpanic=abort"
# export LLVM_PROFILE_FILE="libstackmate-%p.profraw"

cargo test --tests

# grcov . -s . --binary-path target/debug/deps/stackmate-036c699d3ccd39d0 -t lcov --branch --ignore-not-existing -o ./coverage/lcov.info
# genhtml -o ./coverage --show-details --highlight --ignore-errors source --legend ./coverage/lcov.info
# target/debug/deps/stackmate-036c699d3ccd39d0
