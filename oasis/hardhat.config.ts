import { HardhatUserConfig } from "hardhat/config";
import "@nomicfoundation/hardhat-toolbox";
import "@oasisprotocol/sapphire-hardhat";

const config: HardhatUserConfig = {
  solidity: {
    version: "0.8.18",
    optimizer: {
      enabled: true,
      runs: 200,
    }
  },
  networks: {
    sapphire_testnet: {
      url: "https://testnet.sapphire.oasis.dev",
      accounts: process.env.PRIVATE_KEY2
        ? [process.env.PRIVATE_KEY, process.env.PRIVATE_KEY2]
        //0xaEad73A6E2fA0ffD71a285849031486f4004B0EC
        : (process.env.PRIVATE_KEY ? [process.env.PRIVATE_KEY] : []),
      chainId: 0x5aff,
    },
    dev: {
      url: "http://127.0.0.1:8545",
      accounts: [
        // 0x75eCF0d4496C2f10e4e9aF3D4d174576Ee9010E2
        "0x160f52faa5c0aecfa26c793424a04d53cbf23dcad5901ce15b50c2e85b9d6ca7",
        // 0x903a7dce5a26a3f4DE2d157606c2191740Bc4BC9
        "0x0ba685723b47d8e744b1b70a9bea9d4d968f60205385ae9de99865174c1af110"
      ],
      chainId: 0x5afd,
    },
  },
};

export default config;
