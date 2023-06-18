# tee-offchain

This codebase bootstrapped from secret-contract-template.

## Compile
```
make build
```

## Optimize compiled output
```
apt install -y binaryen clang
RUSTFLAGS='-C link-arg=-s' cargo build --release --target wasm32-unknown-unknown --locked
wasm-opt -Oz ./target/wasm32-unknown-unknown/release/*.wasm -o ./contract.wasm
cat ./contract.wasm | gzip -n -9 > ./contract.wasm.gz
```

Main compile output is `contract.wasm.gz` at root directory.

## Upload and instantiate
Either use cli or scripts. 

## Contract
Cosmwasm contract under `src/`

## Schema
The schema file is defined at `src/bin/schema.rs`, Run `cargo run schema` to update schema if msg changes.

## Scripts
All scripts under scripts/ can be run with `npx ts-node scripts/<filename>.ts`

