FROM rust as dev

ARG backtrace=1
ENV RUST_BACKTRACE ${backtrace}

RUN apt update && apt install -y \
      binaryen \
      clang \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/tee-offchain

COPY . .


FROM dev as contract.wasm
RUN make clean
RUN RUSTFLAGS='-C link-arg=-s' cargo build --release --target wasm32-unknown-unknown --locked
RUN cp ./target/wasm32-unknown-unknown/release/*.wasm ./contract.wasm


FROM contract.wasm as contract.wasm.gz
RUN wasm-opt -Oz ./target/wasm32-unknown-unknown/release/*.wasm -o ./contract.wasm
RUN cat ./contract.wasm | gzip -n -9 > ./contract.wasm.gz


FROM scratch as artifacts
COPY --from=contract.wasm /usr/src/tee-offchain/contract.wasm .
COPY --from=contract.wasm.gz /usr/src/tee-offchain/contract.wasm.gz .


#RUN set -eux; \
#      \
#      "RUSTFLAGS='-C link-arg=-s' cargo build --release --target wasm32-unknown-unknown --locked"; \
#      "wasm-opt -Oz ./target/wasm32-unknown-unknown/release/*.wasm -o ./contract.wasm"; \
#      "cat ./contract.wasm | gzip -n -9 > ./contract.wasm.gz"; \
#      "rm -f ./contract.wasm";
