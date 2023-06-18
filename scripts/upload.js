import { SecretNetworkClient, Wallet } from "secretjs";
import * as fs from "fs";
import dotenv from "dotenv";
dotenv.config();

const wallet = new Wallet("miracle cash equal flee lawsuit buffalo victory city relax arrange voice night toilet guilt congress badge reject random fly puzzle bone mystery ugly similar");
// secret1aw22y0wfwhythd06w2c5375q8hrg2jw642d8ks

const contract_wasm = fs.readFileSync(`${process.cwd()}/contract.wasm.gz`);

// Node messaging url
const nodeUrl = "http://localhost:26657";
const secretjs = new SecretNetworkClient({
    chainId: "secretdev-1",
    url: nodeUrl,
    wallet: wallet,
    walletAddress: wallet.address,
  });

// console.log(secretjs);
// let upload_contract = async () => {
//     // let tx = 
//     // console.log(tx);
  
//     // const codeId = Number(
//     //   tx.arrayLog?.find((log) => log.type === "message" && log.key === "code_id")!
//     //     .value
//     // );
  
//     // console.log("codeId: ", codeId);
  
//     // const contractCodeHash = (
//     //   await secretjs.query.compute.codeHashByCodeId({ code_id: codeId })
//     // ).code_hash;
//     // console.log(`Contract hash: ${contractCodeHash}`);
    
//   };
let upload_contract = async () => {
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
const contract_address = "secret1e0e5rv3ktjp0z2fh45z8yhcmpmxemsyhmsswsd";
const contract_code_hash = "9e1172480c3e3e9557e1997ccb3bed45d0cb76266f3c1b2cc4d7c0045a484940";
let try_query_count = async () => {
  
    const my_query = await secretjs.query.compute.queryContract({
      contract_address: contract_address,
      code_hash: contract_code_hash,
      query: { get_state: {} },
    });
  
    console.log(my_query);
};
  
  // try_query_count();

(async () =>{
await try_query_count()

})();

// if(require.main ===module){
// }