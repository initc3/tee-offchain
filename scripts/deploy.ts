import { ethers } from "hardhat";
import * as sapphire from '@oasisprotocol/sapphire-paratime';

async function delay(ms: number = 1000) {
    return new Promise((resolve) => setTimeout(resolve, ms));
}

async function main() {
  let [owner] = await ethers.getSigners();
  let hashMachine = await ethers.deployContract("OffchainHashMachine");
  await hashMachine.waitForDeployment();
  console.log(`Hash machine deployed to ${hashMachine.target}`);
  hashMachine = await ethers.getContractAt("OffchainHashMachine", hashMachine.target, owner);
  console.log("Waiting a little for the contract creation to settle...");
  await delay(10000);
  let state = await hashMachine.getState();
  console.log("State:", state);
  
  console.log("Iterating 10 times...");
  for (let i = 0; i < 10; i++) {
    state = await hashMachine.iterateOffChain(...state);
  }
  console.log("Publishing new state:", state);
  await hashMachine.updateState(...state);
}

// We recommend this pattern to be able to use async/await everywhere
// and properly handle errors.
main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
