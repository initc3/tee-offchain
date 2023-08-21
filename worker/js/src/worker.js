const { Wallet, SecretNetworkClient, MsgExecuteContractResponse, fromUtf8 } = require("secretjs");

const fs = require("fs");

// Load environment variables
require("dotenv").config();
var assert = require('assert');

const setup = async () => {
  // Import wallet from mnemonic phrase
  // Use key created in tutorial #2
  const wallet = new Wallet(process.env.MNEMONIC_1);

  // Create a connection to Secret Network node
  // Pass in a wallet that can sign transactions
  // Docs: https://github.com/scrtlabs/secret.js#secretnetworkclient
  const secretjs = new SecretNetworkClient({
    url: process.env.SECRET_LCD_URL,
    wallet: wallet,
    walletAddress: wallet.address,
    chainId: process.env.SECRET_CHAIN_ID,
  });

  // Upload the wasm of a simple contract
  const wasm = fs.readFileSync("../../contract.wasm");
  console.log("Uploading contract");

  let tx = await secretjs.tx.compute.storeCode(
    {
      sender: wallet.address,
      wasm_byte_code: wasm,
      source: "",
      builder: "",
    },
    {
      gasLimit: 2_000_000,
    }
  );
  assert(tx.code == 0, `transaction failed with code ${tx.code}: ${tx.rawLog}`)

  const codeId = Number(
    tx.arrayLog.find((log) => log.type === "message" && log.key === "code_id")
      .value
  );
  console.log("codeId: ", codeId);

  // contract hash, useful for contract composition
  const contractCodeHash = (await secretjs.query.compute.codeHashByCodeId({code_id: codeId})).code_hash;
  console.log(`Contract hash: ${contractCodeHash}`);

  // Create an instance of the Tee offchain token contract, providing a starting count
  const initMsg = { };
  tx = await secretjs.tx.compute.instantiateContract(
    {
      code_id: codeId,
      sender: wallet.address,
      code_hash: contractCodeHash,
      init_msg: initMsg,
      label: "TEE-OffChain-Token-" + Math.ceil(Math.random() * 10000),
    },
    {
      gasLimit: 100_000,
    }
  );
  assert(tx.code == 0, `transaction failed with code ${tx.code}: ${tx.rawLog}`)

  //Find the contract_address in the logs
  const contractAddress = tx.arrayLog.find(
    (log) => log.type === "message" && log.key === "contract_address"
  ).value;
  console.log(`contractAddress=${contractAddress}`);

  return [codeId, contractAddress, contractCodeHash];
};

const deposit = async (wallet, contractInfo) => {
    codeId = contractInfo[0];
    contractAddress = contractInfo[1];
    contractCodeHash = contractInfo[2];
    // Create a connection to Secret Network node
    // Pass in a wallet that can sign transactions
    // Docs: https://github.com/scrtlabs/secret.js#secretnetworkclient
    const secretjs = new SecretNetworkClient({
      url: process.env.SECRET_LCD_URL,
      wallet: wallet,
      walletAddress: wallet.address,
      chainId: process.env.SECRET_CHAIN_ID,
    });
  
    // Sending deposit request
    console.log(`[user${wallet.address.slice(6,13)}] Sending deposit 100 uscrt`);
  
    tx = await secretjs.tx.compute.executeContract(
      {
        sender: wallet.address,
        contract_address: contractAddress,
        code_hash: contractCodeHash, 
        msg: {submit_deposit: {}},
        sent_funds: [{ amount: "100", denom: "uscrt" }], 
      },
      {
        gasLimit: 100_000,
      }
    );
    assert(tx.code == 0, `transaction failed with code ${tx.code}: ${tx.rawLog}`)
};

const withdraw = async (wallet, contractInfo) => {
    codeId = contractInfo[0];
    contractAddress = contractInfo[1];
    contractCodeHash = contractInfo[2];
    // Create a connection to Secret Network node
    // Pass in a wallet that can sign transactions
    // Docs: https://github.com/scrtlabs/secret.js#secretnetworkclient
    const secretjs = new SecretNetworkClient({
      url: process.env.SECRET_LCD_URL,
      wallet: wallet,
      walletAddress: wallet.address,
      chainId: process.env.SECRET_CHAIN_ID,
    });
  
    // Sending withdraw request
    console.log(`[user${wallet.address.slice(6,13)}] Sending withdraw 50 uscrt`);
  
    tx = await secretjs.tx.compute.executeContract(
      {
        sender: wallet.address,
        contract_address: contractAddress,
        code_hash: contractCodeHash, 
        msg: {submit_withdraw: {amount: "50"}},
        sentFunds: [], 
      },
      {
        gasLimit: 100_000,
      }
    );
    assert(tx.code == 0, `transaction failed with code ${tx.code}: ${tx.rawLog}`)  
};

const transfer = async (wallet, contractInfo, receiverAddr) => {
    codeId = contractInfo[0];
    contractAddress = contractInfo[1];
    contractCodeHash = contractInfo[2];
    // Create a connection to Secret Network node
    // Pass in a wallet that can sign transactions
    // Docs: https://github.com/scrtlabs/secret.js#secretnetworkclient
    const secretjs = new SecretNetworkClient({
      url: process.env.SECRET_LCD_URL,
      wallet: wallet,
      walletAddress: wallet.address,
      chainId: process.env.SECRET_CHAIN_ID,
    });
  
    // Sending transfer request
    console.log(`[user${wallet.address.slice(6,13)}] Sending transfer 50 to user${receiverAddr.slice(6,13)}`);
  
    tx = await secretjs.tx.compute.executeContract(
      {
        sender: wallet.address,
        contract_address: contractAddress,
        code_hash: contractCodeHash, 
        msg: {submit_transfer: {to: receiverAddr, memo: "memo", amount: "50"}},
        sentFunds: [], 
      },
      {
        gasLimit: 100_000,
      }
    );
    assert(tx.code == 0, `transaction failed with code ${tx.code}: ${tx.rawLog}`)
  
};

const balance = async (wallet, contractInfo, viewingKey) => {
    codeId = contractInfo[0];
    contractAddress = contractInfo[1];
    contractCodeHash = contractInfo[2];
    const secretjs = new SecretNetworkClient({
      url: process.env.SECRET_LCD_URL,
      wallet: wallet,
      walletAddress: wallet.address,
      chainId: process.env.SECRET_CHAIN_ID,
    });

  const balance = await secretjs.query.compute.queryContract({
    contract_address: contractAddress,
    code_hash: contractCodeHash,
    query: { get_balance: {address: wallet.address, key: viewingKey} },
  });

  console.log(`[user${wallet.address.slice(6,13)}] balance=${balance}`);

  
};

const setupViewingKey = async (wallet, contractInfo) => {
    codeId = contractInfo[0];
    contractAddress = contractInfo[1];
    contractCodeHash = contractInfo[2];
    const secretjs = new SecretNetworkClient({
      url: process.env.SECRET_LCD_URL,
      wallet: wallet,
      walletAddress: wallet.address,
      chainId: process.env.SECRET_CHAIN_ID,
    });
  
    tx = await secretjs.tx.compute.executeContract(
      {
        sender: wallet.address,
        contract_address: contractAddress,
        code_hash: contractCodeHash, 
        msg:  { create_viewing_key: { entropy: wallet.address + "-" + Math.ceil(Math.random() * 10000) } },
        sent_funds: [], 
      },
      {
        gasLimit: 100_000,
      }
    );
    const viewingKey = JSON.parse(fromUtf8(MsgExecuteContractResponse.decode(tx.data[0]).data)).key;  

    console.log(`[user${wallet.address.slice(6,13)}] viewingKey ${viewingKey}`);

    tx = await secretjs.tx.compute.executeContract(
        {
        sender: wallet.address,
        contract_address: contractAddress,
        code_hash: contractCodeHash, 
        msg: { set_viewing_key: {key: viewingKey}},
        sentFunds: [], 
        },
        {
        gasLimit: 100_000,
        }
    );
    assert(tx.code == 0, `transaction failed with code ${tx.code}: ${tx.rawLog}`)
    return viewingKey;
  
};


const getState = async (wallet, contractInfo) => {
    codeId = contractInfo[0];
    contractAddress = contractInfo[1];
    contractCodeHash = contractInfo[2];
    const secretjs = new SecretNetworkClient({
      url: process.env.SECRET_LCD_URL,
      wallet: wallet,
      walletAddress: wallet.address,
      chainId: process.env.SECRET_CHAIN_ID,
    });

  const state = await secretjs.query.compute.queryContract({
    contract_address: contractAddress,
    code_hash: contractCodeHash,
    query: { get_checkpoint: {} },
  });

  return state;  
};

const processNext = async (wallet, contractInfo, inState) => {
    codeId = contractInfo[0];
    contractAddress = contractInfo[1];
    contractCodeHash = contractInfo[2];
    // console.log(`processNext instate=`, inState);
    const secretjs = new SecretNetworkClient({
      url: process.env.SECRET_LCD_URL,
      wallet: wallet,
      walletAddress: wallet.address,
      chainId: process.env.SECRET_CHAIN_ID,
    });

  const outState = await secretjs.query.compute.queryContract({
    contract_address: contractAddress,
    code_hash: contractCodeHash,
    query: { process_next: {cipher: inState} },
  });
  assert(!String(outState).includes("error"), `transaction failed with error ${outState}`)
  // console.log(`processNext outState=`, outState);

  return outState;  
};

const commitState = async (wallet, contractInfo, inState) => {
    codeId = contractInfo[0];
    contractAddress = contractInfo[1];
    contractCodeHash = contractInfo[2];
    // console.log(`commitState inState=`, inState);
    const secretjs = new SecretNetworkClient({
      url: process.env.SECRET_LCD_URL,
      wallet: wallet,
      walletAddress: wallet.address,
      chainId: process.env.SECRET_CHAIN_ID,
    });

    // Sending commit 
    console.log(`[worker${wallet.address.slice(6,11)}] Sending commit`);
  
    tx = await secretjs.tx.compute.executeContract(
      {
        sender: wallet.address,
        contract_address: contractAddress,
        code_hash: contractCodeHash, 
        msg: {commit_response: {cipher: inState}},
        sentFunds: [], 
      },
      {
        gasLimit: 100_000,
      }
    );
    assert(tx.code == 0, `transaction failed with code ${tx.code}: ${tx.rawLog}`)
};

const writeCheckpoint = async (wallet, contractInfo, inState) => {
    codeId = contractInfo[0];
    contractAddress = contractInfo[1];
    contractCodeHash = contractInfo[2];
    // console.log(`writeCheckpoint inState=`, inState);
    const secretjs = new SecretNetworkClient({
      url: process.env.SECRET_LCD_URL,
      wallet: wallet,
      walletAddress: wallet.address,
      chainId: process.env.SECRET_CHAIN_ID,
    });
  
    // Sending write checkpoint
    console.log(`[worker${wallet.address.slice(6,11)}] Sending write checkpoint`);
  
    tx = await secretjs.tx.compute.executeContract(
      {
        sender: wallet.address,
        contract_address: contractAddress,
        code_hash: contractCodeHash, 
        msg: {write_checkpoint: {cipher: inState}},
        sentFunds: [], 
      },
      {
        gasLimit: 100_000,
      }
    );
    assert(tx.code == 0, `transaction failed with code ${tx.code}: ${tx.rawLog}`)  
};

const worker = async (wallet_worker, contractInfo, num_states, curr_state, timeout) => {
  getState(wallet_worker, contractInfo).then((state_0) => {
      processNext(wallet_worker, contractInfo, state_0).then((state_1) => {
      commitState(wallet_worker, contractInfo, state_1.request_cipher).then(() => {
        writeCheckpoint(wallet_worker, contractInfo, state_1.checkpoint_cipher).then(() => {
          if (1 == curr_state) {
            setTimeout(worker, timeout, wallet_worker, contractInfo, num_states, num_states, timeout);                
          } else {
            worker(wallet_worker, contractInfo, num_states, curr_state-1, timeout);
          }
        }).catch((reason) => {
          console.log("error in writeCheckpoint", reason);
        });
      }).catch((reason) => {
        console.log("error in commitState", reason);
      });
    }).catch((reason) => {
      if (String(reason).includes("seqno less than number of requests")){ //There were no new requests to process
        setTimeout(worker, timeout, wallet_worker, contractInfo, num_states, curr_state);   
      } else {
        console.log("error in processNext", reason);
      }
  });
}).catch((reason) => {
  console.log("error in getCheckpoint", reason);
});
}


const users_balance = async(wallet_2, wallet_3, contractInfo, vks) => {
  balance(wallet_2, contractInfo, vks[0]).then(() => {  
    balance(wallet_3, contractInfo, vks[1]).then(() => { 
    }).catch((reason) => {
      console.log("error in balance user 3", reason);
    })
  }).catch((reason) => {
    console.log("error in balance user 2", reason);
  })
}

const users_withdraw = async(wallet_2, wallet_3, contractInfo, vks) => {
  withdraw(wallet_2, contractInfo).then(() => {  
    withdraw(wallet_3, contractInfo).then(() => {  
    }).catch((reason) => {
      console.log("error in withdraw user 3", reason);
    });
  }).catch((reason) => {
    console.log("error in withdraw user 2", reason);
  });
}

const wallet_worker = new Wallet(process.env.MNEMONIC_1);

const wallet_2 = new Wallet(process.env.MNEMONIC_2);
const wallet_3 = new Wallet(process.env.MNEMONIC_3);

const tm = 5000;



const run = async () => {
  const contractInfo = await setup();
  await deposit(wallet_2, contractInfo);
  await deposit(wallet_3, contractInfo);
  const vk2 = await setupViewingKey(wallet_2, contractInfo);
  const vk3 = await setupViewingKey(wallet_3, contractInfo);
  await balance(wallet_2, contractInfo, vk2);
  await balance(wallet_3, contractInfo, vk3);
  
  console.log(`*****************starting worker${wallet_worker.address.slice(6,11)} that processes 2 transactions every ${tm} ms*****************`);
  worker(wallet_worker, contractInfo, 2 , 2, tm);

  setTimeout(users_balance, tm*5 ,wallet_2, wallet_3, contractInfo, [vk2, vk3]);

  setTimeout(transfer, tm*10, wallet_2, contractInfo, wallet_3.address);
  setTimeout(users_balance, tm*15 ,wallet_2, wallet_3, contractInfo, [vk2, vk3]);

  setTimeout(users_withdraw, tm*20, wallet_2, wallet_3, contractInfo);
  setTimeout(users_balance, tm*25, wallet_2, wallet_3, contractInfo, [vk2, vk3]);

};

run().then(() => {
}).catch((reason) => {
  console.log("error in run", reason);
});
