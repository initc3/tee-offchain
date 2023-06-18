# tee-offchain

## Unit tests
```
cargo test
```

## Build and deploy the contract on a local net
Launch the local net
```
docker run -it -p 9091:9091 -p 26657:26657 -p 1317:1317 -p 5000:5000 \
  --name localsecret ghcr.io/scrtlabs/localsecret:latest
```

Build and optmize the contract via a docker container with:
```
make contract
```
This will output a `contract.wasm.gz` file ready to be deployed.

Store the contract on chain
```
secretcli tx compute store contract.wasm.gz --gas 5000000 --from <your-address> --chain-id secretdev-1
```

Check
```
secretcli query compute list-code
```

Instantiate the contract
```
secretcli tx compute instantiate 1 {} --from <your-address> --label rollupContract -y
```

Check
```
secretcli query compute list-contract-by-code 1
```


## Contributing
File issues & pull requests as you wish!

1. Clone the repo
2. Create a branch
4. Make a pull request
5. Ask your peers to review
6. Wait for the tests to pass
7. Once reviewed and the tests have passed, maintainers will rebase & merge

You can think of a pull request like a block proposal in a blockchain.
The merging of the block in a blockchain is achieved via a consensus protocol,
whereas in collaborative software development it is achieved via a combination
of automated processes such as continous integration, and code reviews by humans.
**Since humans are involved, you can think of it like a sport team or a music band,
with whom you are playing.**

Will very loosely attempt to follow ZeroMQ's RFC 42/C4:
the [Collective Code Construction Contract][c4].

At the very least, let's try to follow:

> **2.3. Patch Requirements**
>
> 2. A patch SHOULD be a minimal and accurate answer to exactly one identified and
     agreed problem.

> 6. A patch MUST compile cleanly and pass project self-tests on at least the
     principal target platform.

> **2.5. Branches and Releases**
>
> 1. The project SHALL have one branch ("main") that always holds the latest
     in-progress version and SHOULD always build.

[c4]: https://rfc.zeromq.org/spec/42/
