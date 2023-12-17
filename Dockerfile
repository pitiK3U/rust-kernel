FROM rust:latest

RUN apt-get update && apt-get install -y nasm gdb rust-gdb

RUN rustup override set nightly && rustup component add rust-src --toolchain nightly-x86_64-unknown-linux-gnu


RUN rustup component add llvm-tools-preview && cargo install cargo-binutils
  