import { secretjs } from "./client";

export async function getBalance(address: string, denom: string) {
    const res = await secretjs.query.bank.balance(
        {
            address: address,
            denom: denom
        }
    )
    console.log(res);
}

if(require.main === module){
    (async () => {
        await getBalance("secret1ap26qrlp8mcq2pg6r47w43l0y8zkqm8a450s03", "uscrt")
    })();
}