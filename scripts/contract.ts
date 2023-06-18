import { SecretNetworkClient, Wallet } from "secretjs";
import * as fs from "fs";
import dotenv from "dotenv";
import { secretjs } from "./client";
dotenv.config();

const wallet = new Wallet("miracle cash equal flee lawsuit buffalo victory city relax arrange voice night toilet guilt congress badge reject random fly puzzle bone mystery ugly similar");
// secret1aw22y0wfwhythd06w2c5375q8hrg2jw642d8ks

const contract_wasm = fs.readFileSync(`${process.cwd()}/contract.wasm.gz`);

export async function upload_contract() {
  const tx = await secretjs.tx.compute.storeCode(
    {
      sender: wallet.address,
      wasm_byte_code: contract_wasm,
      source: "",
      builder: "",
    },
    {
      gasLimit: 4_000_000,
    }
);
console.log(tx); 
}

export async function instantiate_contract(codeId: number, codeHash: string, contractLabel: string) {
   // Replace init msg with the 
   const initMsg = { };
   let tx = await secretjs.tx.compute.instantiateContract(
     {
       code_id: codeId,
       sender: wallet.address,
       code_hash: codeHash,
       init_msg: initMsg,
       label: contractLabel
     },
     {
       gasLimit: 400_000,
     }
   );
    console.log(tx);
}

const codeId = 1;
const contract_address = "secret1e0e5rv3ktjp0z2fh45z8yhcmpmxemsyhmsswsd";
const contract_code_hash = "9e1172480c3e3e9557e1997ccb3bed45d0cb76266f3c1b2cc4d7c0045a484940";
const contractLabel = "iterativeHashContract";
if(require.main  === module) {
  (async () => {
    await instantiate_contract(codeId, contract_code_hash, contractLabel);
  })();
}
