FROM rust:1.69.0

WORKDIR /home

RUN rustup target add wasm32-wasi

ADD Cargo.toml /home/Cargo.toml
ADD lib.rs /home/src/lib.rs
RUN cargo build --target wasm32-wasi --release