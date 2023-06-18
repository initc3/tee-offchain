import { ethers } from "hardhat";
import * as sapphire from '@oasisprotocol/sapphire-paratime';

async function delay(ms: number = 1000) {
    return new Promise((resolve) => setTimeout(resolve, ms));
}

async function process(checkpoint, nonce, token, account, seq, responsesLen) {
  let response, responseNonce;
  for (let i = seq + 1n;; i++) {
    console.log(`Processing next action ${i}...`);
    try {
        let result = await token.processNext(checkpoint, nonce);
        //console.log(result);
        [checkpoint, nonce, response, responseNonce] = result;
        if (i > responsesLen) {
            console.log("Committing action...");
            let commitResult = await token.connect(account).commitResponse(response, responseNonce);
            let receipt = await commitResult.wait();
            console.log(receipt);
            if (receipt.status == 0) {
                console.log("Commit transaction reverted. That means we're done");
                break;
            }
        } else {
            console.log("Response already computed for this index...");
        }
    } catch (e) {
        console.log("Done", e);
        break;
    }
  }
  
  /*console.log("Writing the checkpoint to contract state...");
  let r = await token.writeCheckpoint(checkpoint, nonce);
  await r.wait();*/
}

async function main() {
  let [owner, owner2] = await ethers.getSigners();
  let token = await ethers.getContractAt("VeiledToken", "0x06a46c52f88d870A2DB9b02f6EC5d7924b8C5Ced", owner);
  console.log(`Accessing VeiledToken at ${token.target}`);
  token = await ethers.getContractAt("VeiledToken", token.target, owner);
  let checkpoint = await token.getCheckpoint();
  let checkpointSeq = await token.checkpointSeq();
  console.log("Current checkpoint sequence number:", checkpointSeq);
  console.log("Current checkpoint:", checkpoint);
  
  console.log();
  
  const depositAmt = ethers.parseEther("0.1");
  const transferAmt = depositAmt / 2n;
  let depositTx = await token.connect(owner).requestDeposit({ value: depositAmt });
  await delay(5000);
  await depositTx.wait();
  
  console.log(`Depositing ${ethers.formatEther(depositAmt)} ROSE to the token...`);
  console.log("Recomputing the token state off chain...");
  let responsesLen;
  responsesLen = (await token.getResponses()).length;
  checkpointSeq = await token.checkpointSeq();
  console.log("Responses:", responsesLen, "Checkpoint seq:", checkpointSeq);
  await process(checkpoint[0], checkpoint[1], token, owner, checkpointSeq, responsesLen);
  //let responsesLen = 3n;
  //console.log(checkpointSeq + 1 - responsesLen, "to go");
  
  console.log(`Now let's transfer some of this stuff. Transferring ${ethers.formatEther(transferAmt)} ROSE to some other account...`);
  await token.connect(owner).requestTransfer(owner2.address, transferAmt, "0x523322ef8180a10bf2c8e6f1681bb55fab82570dc98312c010a92335be700000");
  await delay(5000);
  
  let nonce;
  [checkpoint, nonce] = await token.getCheckpoint();
  responsesLen = (await token.getResponses()).length;
  checkpointSeq = await token.checkpointSeq();
  await process(checkpoint, nonce, token, owner, checkpointSeq, responsesLen);
  
  console.log(`Creating a withdraw request for account 2...`);
  await token.connect(owner2).requestWithdraw(transferAmt);
  
  [checkpoint, nonce] = await token.getCheckpoint();
  responsesLen = (await token.getResponses()).length;
  checkpointSeq = await token.checkpointSeq();
  await process(checkpoint, nonce, token, owner2, checkpointSeq, responsesLen);
}

// We recommend this pattern to be able to use async/await everywhere
// and properly handle errors.
main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
