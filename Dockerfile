FROM rust

ARG backtrace=1
ENV RUST_BACKTRACE ${backtrace}

RUN rustup target add wasm32-unknown-unknown

WORKDIR /usr/src/tee-offchain

COPY . .
