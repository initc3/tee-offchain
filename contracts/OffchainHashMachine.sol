// SPDX-License-Identifier: MIT
pragma solidity >=0.8.0 <0.9.0;

import "@oasisprotocol/sapphire-contracts/contracts/Sapphire.sol";

// Uncomment this line to use console.log
// import "hardhat/console.sol";

contract OffchainHashMachine {
    bytes32 seed;
    
    constructor() {
        seed = bytes32(Sapphire.randomBytes(32, ""));
    }
}
