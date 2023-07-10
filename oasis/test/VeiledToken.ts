import {
  time,
  loadFixture,
} from "@nomicfoundation/hardhat-toolbox/network-helpers";
import { anyValue } from "@nomicfoundation/hardhat-chai-matchers/withArgs";
import { expect } from "chai";
import { ethers } from "hardhat";
import * as sapphire from '@oasisprotocol/sapphire-paratime';

enum RequestType {
    TRANSFER,
    DEPOSIT,
    WITHDRAW
};

describe("VeiledToken", function () {
  async function deploy() {
    // Contracts are deployed using the first signer/account by default
    const [owner, account2] = await ethers.getSigners();

    const VeiledToken = await ethers.getContractFactory("VeiledToken");
    const veiledToken = await VeiledToken.deploy();

    return { veiledToken, owner, account2 };
  }

  describe("Deployment", function () {
    it("Should deploy", async function () {
      const { veiledToken } = await deploy();
    });
    
    it("Should write a checkpoint", async function () {
      const { veiledToken, owner } = await deploy();
      let [checkpoint1, nonce1] = await veiledToken.getCheckpoint();
      await veiledToken.writeCheckpoint(checkpoint1, nonce1);
    });
    
    it("Should commit a deposit and a withdrawal", async function () {
      const { veiledToken, owner, account2 } = await deploy();
      const depositAmt = ethers.parseEther("0.1");
      const transferAmt = depositAmt / 2n;
      await expect(veiledToken.connect(owner).requestDeposit({
        value: depositAmt
      }))
        .to.emit(veiledToken, "RequestSubmitted")
        .withArgs(0, RequestType.DEPOSIT, owner.address);
      
      let [checkpoint1, nonce1] = await veiledToken.getCheckpoint();
      let [checkpoint2, nonce2, response, responseNonce] = await veiledToken.processNext(checkpoint1, nonce1);
      await expect(veiledToken.commitResponse(response, responseNonce))
        .to.emit(veiledToken, "ResponseCommitted")
        .withArgs(1, [1, true, depositAmt, ""]);
      
      // Transfer
      await expect(veiledToken.connect(owner).requestTransfer(account2.address, transferAmt, "0x1234000000000000000000000000000000000000000000000000000000000000"))
        .to.emit(veiledToken, "RequestSubmitted")
        .withArgs(1, RequestType.TRANSFER, owner.address);
      let [checkpoint3, nonce3, response2, responseNonce2] = await veiledToken.processNext(checkpoint2, nonce2);
      await expect(veiledToken.connect(owner).commitResponse(response2, responseNonce2))
        .to.emit(veiledToken, "ResponseCommitted")
        .withArgs(2, [2, true, 0, "transfer ok"]);
      
      // Withdrawal
      await expect(veiledToken.connect(account2).requestWithdraw(account2.address, transferAmt))
        .to.emit(veiledToken, "RequestSubmitted")
        .withArgs(2, RequestType.WITHDRAW, account2.address);
      let [checkpoint4, nonce4, response3, responseNonce3] = await veiledToken.processNext(checkpoint3, nonce3);
      let balanceBefore = await ethers.getBalance(account2.address);
      await expect(veiledToken.connect(account2).commitResponse(response3, responseNonce3))
        .to.emit(veiledToken, "ResponseCommitted")
        .withArgs(3, [3, true, transferAmt, ""]);
      
      expect(await ethers.getBalance(account2.address) > balanceBefore).to.be.true;
    });
  });
});
