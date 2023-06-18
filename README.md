# Sample Hardhat Project

## Create the dev network:
```
docker run -it -p8545:8545 -p8546:8546 ghcr.io/oasisprotocol/sapphire-dev -to "0x72A6CF1837105827077250171974056B40377488"
```

## Run test cases
```shell
npx hardhat test --network dev
```
