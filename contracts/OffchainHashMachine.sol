// SPDX-License-Identifier: MIT
pragma solidity >=0.8.0 <0.9.0;

import "@oasisprotocol/sapphire-contracts/contracts/Sapphire.sol";

// Uncomment this line to use console.log
// import "hardhat/console.sol";

contract OffchainHashMachine {
    // This contract will maintain the invariant that
    // curhash == Hash(Hash(.... <counter times>...Hash(seed))...)
    uint256 counter = 0;
    // Current hash
    bytes32 curHash = bytes32("seed");
    // Private key used to create MACs
    bytes32 key;
    
    constructor() {
        bytes memory randomBytes = Sapphire.randomBytes(32, "");
        require(randomBytes.length == 32, "Incorrect random bytes length. Not Sapphire?");
        require(uint256(bytes32(randomBytes)) > 0, "randomBytes returned 0. Not Sapphire?");
        key = bytes32(randomBytes);
        curHash = key;
    }
    
    function genMAC(bytes memory data) private view returns (bytes32 mac) {
        mac = keccak256(abi.encodePacked(key, data));
    }
    
    function getState() public view returns (uint256, bytes32, bytes32) {
        return (counter, curHash, genMAC(abi.encodePacked(counter, curHash)));
    }
    
    function iterateOnChain() public {
    	counter += 1;
    	curHash = keccak256(abi.encodePacked(curHash));
    }
    
    function updateState(uint256 newCounter, bytes32 newHash, bytes32 mac) public {
		// This is mutable, this runs on-chain
		// Check a MAC on the newcounter/newhash pair
		require(genMAC(abi.encodePacked(newCounter, newHash)) == mac, "Invalid MAC");
		require(newCounter > counter, "Count already reached");
		counter = newCounter;
		curHash = newHash;
    }
    
    function iterateOffChain(uint256 prevCounter, bytes32 prevHash, bytes32 prevMac) public view returns (uint256, bytes32, bytes32) {
        require(genMAC(abi.encodePacked(prevCounter, prevHash)) == prevMac, "Invalid MAC");
        uint256 newCounter = counter + 1;
        bytes32 newHash = keccak256(abi.encodePacked(prevHash));
        bytes32 newMac = genMAC(abi.encodePacked(newCounter, newHash));
        return (newCounter, newHash, newMac);
	}

}
