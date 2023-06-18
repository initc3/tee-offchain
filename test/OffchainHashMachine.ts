import {
  time,
  loadFixture,
} from "@nomicfoundation/hardhat-toolbox/network-helpers";
import { anyValue } from "@nomicfoundation/hardhat-chai-matchers/withArgs";
import { expect } from "chai";
import { ethers } from "hardhat";
import * as sapphire from '@oasisprotocol/sapphire-paratime';

describe("OffchainHashMachine", function () {
  // We define a fixture to reuse the same setup in every test.
  // We use loadFixture to run this setup once, snapshot that state,
  // and reset Hardhat Network to that snapshot in every test.
  async function deploy() {
    // Contracts are deployed using the first signer/account by default
    const [owner] = await ethers.getSigners();
    //owner = sapphire.wrap(owner);

    const HashMachine = await ethers.getContractFactory("OffchainHashMachine");
    const hashMachine = await HashMachine.deploy();

    return { hashMachine, owner };
  }

  describe("Deployment", function () {
    it("Should deploy", async function () {
      const { hashMachine } = await deploy();
      console.log(await hashMachine.getState());
    });
    
    it("Should update to a new state", async function () {
      const { hashMachine } = await deploy();
      const [counterOld, hashOld, macOld] = await hashMachine.getState();
      // Calculate new state off-chain
      let [counterNew, hashNew, macNew] = await hashMachine.iterateOffChain(counterOld, hashOld, macOld);
      for (let i = 0; i < 5; i++) {
        [counterNew, hashNew, macNew] = await hashMachine.iterateOffChain(counterNew, hashNew, macNew);
      }
      await hashMachine.updateState(counterNew, hashNew, macNew);
      // Verify that the state was changed
      const [counterLatest, hashLatest, macLatest] = await hashMachine.getState();
      expect(counterLatest).to.equal(counterNew);
      expect(hashLatest).to.equal(hashNew);
      expect(macLatest).to.equal(macNew);
    });
  });
});
