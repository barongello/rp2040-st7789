#!/bin/zsh

rustup update
cargo install cargo-binutils
cargo install elf2uf2-rs
cargo install flip-link
rustup component add llvm-tools-preview
rustup target add thumbv6m-none-eabi
