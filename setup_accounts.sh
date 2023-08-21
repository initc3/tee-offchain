set -x
set -e

secretcli config node http://localhost:26657
secretcli config chain-id secretdev-1
secretcli config keyring-backend test
secretcli config output json


MNEMONIC_1="fabric toe special change advice december shiver recall shoe jar glide catalog skin october vehicle physical increase lyrics quote name fine border portion fancy"
ADDRESS_1='secret1ld9ak4qfn2t2fg3x3yz59lu7rg7tpw7gae7gqj'
MNEMONIC_2='merge coast limb solution body truck push oppose black excess inflict electric assume rescue mean project rice pig liar table siege magic silk slush'
ADDRESS_2="secret1a57rwyazlu09vvcsh5602jchmfhaxnrs2rq0yu"
ADDRESS_3='secret1tah2fd6cltk8e70epdzv9d9mrre6qypsd8gcjx'
MNEMONIC_3="first wood anchor sick decrease kitten wall fossil logic injury tuition cinnamon drill camera mother text oxygen filter hurt slender ostrich surface shell soldier"


eval TXHASH_1=$(curl http://localhost:5000/faucet?address=$ADDRESS_1 | jq .txhash )
sleep 5
secretcli q tx $TXHASH_1 | jq .code
secretcli query bank balances $ADDRESS_1 | jq .

eval TXHASH_2=$(curl http://localhost:5000/faucet?address=$ADDRESS_2 | jq .txhash )
sleep 5
secretcli q tx $TXHASH_2 | jq .code
secretcli query bank balances $ADDRESS_2 | jq .

eval TXHASH_3=$(curl http://localhost:5000/faucet?address=$ADDRESS_3 | jq .txhash )
sleep 5
secretcli q tx $TXHASH_3 | jq .code
secretcli query bank balances $ADDRESS_3 | jq .

