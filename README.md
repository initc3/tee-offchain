# Off-chain private tokens
An implementation of off-chain private tokens in Oasis Sapphire.

Requirements:
* Node
* Docker

## Setup
First, install all required packages with `npm`:
```
npm install
```

### Create a dev network:
```
docker run -it -p8545:8545 -p8546:8546 ghcr.io/oasisprotocol/sapphire-dev -to "0x72A6CF1837105827077250171974056B40377488"
```

### Run test cases
```shell
npx hardhat test --network dev
```

Test the deployment of the off-chain hash machine on the public Sapphire testnet:
```
PRIVATE_KEY=<funded private key> npx hardhat run scripts/deploy.ts --network sapphire_testnet
```
