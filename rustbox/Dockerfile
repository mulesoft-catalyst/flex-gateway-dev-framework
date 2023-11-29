FROM rust:1.69.0

WORKDIR /home

RUN rustup target add wasm32-wasi

ADD Cargo.toml /home/Cargo.toml
ADD lib.rs /home/src/lib.rs
RUN echo "Starting cargo build..." && \
    CARGO_NET_GIT_FETCH_WITH_CLI=true && \
    cargo build --target wasm32-wasi --release && \
    echo "Cargo build completed."