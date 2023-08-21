# tee-offchain

## Unit tests
```
cargo test
```

## Build and deploy the contract on a local net
Launch the local net
```
make start-server
```

In a seperate terminal Setup test accounts
```
./setup_accounts.sh
```

Build and optmize the contract via a docker container with:
```
make build
```
This will output a `contract.wasm.gz` file ready to be deployed.


Run worker script
```
cd worker/js
npm install
npm run worker
```

Example Output
```
Uploading contract
codeId:  52
Contract hash: 69c324044c4362862657d9a48368bae6a75af9a3bce1f8831dd148c4e7cb1c2b
contractAddress=secret1fse00hs0clpgkaz83rkc0rtglr04uqpf6ctv5y
[user1a57rwy] Sending deposit 100 uscrt
[user1tah2fd] Sending deposit 100 uscrt
[user1a57rwy] viewingKey api_key_8DrgKPL1S4ksEOBOaqE2B06pkyKrxSeyLw99UBgxX5c=
[user1tah2fd] viewingKey api_key_8LioLjry849VFUVY+3wD5tNBkVnotyOiNM2JM1546+w=
[user1a57rwy] balance=0
[user1tah2fd] balance=0
*****************starting worker1ld9a that processes 2 transactions every 5000 ms*****************
[worker1ld9a] Sending commit
[worker1ld9a] Sending write checkpoint
[worker1ld9a] Sending commit
[worker1ld9a] Sending write checkpoint
[user1a57rwy] balance=100
[user1tah2fd] balance=100
[user1a57rwy] Sending transfer 50 to user1tah2fd
[worker1ld9a] Sending commit
[worker1ld9a] Sending write checkpoint
[user1a57rwy] balance=50
[user1tah2fd] balance=150
[user1a57rwy] Sending withdraw 50 uscrt
[user1tah2fd] Sending withdraw 50 uscrt
[worker1ld9a] Sending commit
[worker1ld9a] Sending write checkpoint
[worker1ld9a] Sending commit
[worker1ld9a] Sending write checkpoint
[user1a57rwy] balance=0
[user1tah2fd] balance=100
```