#!/bin/sh
printf "=== Runing Language Tests ==="
printf "\n=== Building ===\n"
cargo build --release
printf "\n=== Print Test ===\n"
./target/release/rlok ./lang_tests/test_print.lox
printf "\n=== Scopes Test ===\n"
./target/release/rlok ./lang_tests/test_scopes.lox
