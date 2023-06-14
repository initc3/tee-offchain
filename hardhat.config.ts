import { HardhatUserConfig } from "hardhat/config";
import "@nomicfoundation/hardhat-toolbox";

const config: HardhatUserConfig = {
  solidity: "0.8.18",
  networks: {
    local: {
      url: "http://localhost:8545",
      // 0x3AA6a13b19f111C2925850253e86548164477248
      accounts: ["0xdf1d2db3bf5b6f637b0db2f2076d6eca96d3a8da29f4f31cbb6ae262d96f26c9"],
    }
  }
};

export default config;
