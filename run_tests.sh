#!/bin/sh
printf "=== Runing Language Tests ==="
printf "\n=== Building ===\n"
cargo build --release
printf "\n=== Print Test ===\n"
./target/release/rlok ./lang_tests/test_print.lox
printf "\n=== Variables Test ===\n"
./target/release/rlok ./lang_tests/test_variables.lox
printf "\n=== Scopes Test ===\n"
./target/release/rlok ./lang_tests/test_scopes.lox
printf "\n=== If/else statement Test ===\n"
./target/release/rlok ./lang_tests/test_if_statements.lox
printf "\n=== and/or Test ===\n"
./target/release/rlok ./lang_tests/test_and_or.lox
