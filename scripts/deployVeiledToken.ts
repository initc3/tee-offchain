import { ethers } from "hardhat";
import * as sapphire from '@oasisprotocol/sapphire-paratime';

async function delay(ms: number = 1000) {
    return new Promise((resolve) => setTimeout(resolve, ms));
}

async function main() {
  let [owner] = await ethers.getSigners();
  let token = await ethers.deployContract("VeiledToken");
  await token.waitForDeployment();
  console.log(`VeiledToken deployed to ${token.target}`);
  token = await ethers.getContractAt("VeiledToken", token.target, owner);
  console.log("Waiting a little for the contract creation to settle...");
  await delay(10000);
  let checkpoint = await token.getCheckpoint();
  console.log("First checkpoint:", checkpoint);
  
}

// We recommend this pattern to be able to use async/await everywhere
// and properly handle errors.
main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
