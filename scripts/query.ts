import { secretjs } from "./client";

const contract_address = "secret1e0e5rv3ktjp0z2fh45z8yhcmpmxemsyhmsswsd";
const contract_code_hash = "9e1172480c3e3e9557e1997ccb3bed45d0cb76266f3c1b2cc4d7c0045a484940";

export async function get_state() {
    const my_query = await secretjs.query.compute.queryContract({
        contract_address: contract_address,
        code_hash: contract_code_hash,
        query: { get_state: {} },
      });
    
      console.log(my_query);
}

if(require.main === module){
    (async () => {
        await get_state();
    })();
}