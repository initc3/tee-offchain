// SPDX-License-Identifier: MIT
pragma solidity >=0.8.0 <0.9.0;

import "@oasisprotocol/sapphire-contracts/contracts/Sapphire.sol";

struct AccountState {
    address addr;
    uint256 balance;
}

enum RequestType {
    TRANSFER,
    DEPOSIT,
    WITHDRAW
}

struct Request {
    RequestType rtype;
    address from;
    // Equal to address(0) for deposits and withdrawals
    address to;
    uint256 amount;
    bytes32 memo;
}

struct Response {
    uint256 seqNum;
    bool status;
    uint256 amount;
    string response;
}

contract VeiledToken {
    // Private key used to verify state
    bytes32 key;
    
    // Checkpoint public state
    AccountState[] public checkpoint;
    uint256 public checkpointSeq;
    
    // List of pending commands
    // Only the type and sender are publicly visible
    Request[] requests;
    
    // Processed responses
    // Length will never be greater than the size of `requests`
    Response[] public responses;
    
    // Events
    event RequestSubmitted(uint256 indexed seqNum, RequestType indexed rtype, address indexed from);
    event ResponseCommitted(uint256 indexed seqNum, Response response);
    
    constructor() {
        bytes memory randomBytes = Sapphire.randomBytes(32, "");
        require(randomBytes.length == 32, "Incorrect random bytes length. Not Sapphire?");
        require(uint256(bytes32(randomBytes)) > 0, "randomBytes returned 0. Not Sapphire?");
        key = bytes32(randomBytes);
    }
    
    function getRequest(uint256 seqNum) public view returns (RequestType, address) {
        return (requests[seqNum].rtype, requests[seqNum].from);
    }
    
    /*
        Submitting requests
    */
    function requestDeposit() payable public {
        requests.push(Request(RequestType.DEPOSIT, msg.sender, address(0), msg.value, bytes32(0)));
        emit RequestSubmitted(requests.length - 1, RequestType.DEPOSIT, msg.sender);
    }
    
    function requestTransfer(address to, uint256 amount, bytes32 memo) public {
        requests.push(Request(RequestType.TRANSFER, msg.sender, to, amount, memo));
        emit RequestSubmitted(requests.length - 1, RequestType.TRANSFER, msg.sender);
    }
    
    function requestWithdraw(uint256 amount) public {
        requests.push(Request(RequestType.WITHDRAW, msg.sender, address(0), amount, bytes32(0)));
        emit RequestSubmitted(requests.length - 1, RequestType.WITHDRAW, msg.sender);
    }
    
    /*
        Checkpoints
    */
    function getCheckpoint() public view returns (bytes memory, bytes32) {
        bytes32 randomNonce = bytes32(Sapphire.randomBytes(32, ""));
        bytes memory ciphertext = Sapphire.encrypt(key, randomNonce, abi.encode(checkpointSeq, checkpoint), "checkpoint");
        return (ciphertext, randomNonce);
    }
    
    function writeCheckpoint(bytes calldata ciphertext, bytes32 nonce) public {
        // The encryption handles authentication
        (uint256 newSeq, AccountState[] memory newCheckpoint) = abi.decode(Sapphire.decrypt(key, nonce, ciphertext, "checkpoint"), (uint256, AccountState[]));
        require(newSeq >= checkpointSeq, "Checkpoint sequence already defined");
        checkpointSeq = newSeq;
        
        // Essentially, checkpoint = newCheckpoint;
        delete checkpoint;
        for (uint256 i = 0; i < newCheckpoint.length; i++) {
            checkpoint.push(newCheckpoint[i]);
        }
    }
    
    /*
        Off-chain computation
    */
    function processNext(bytes calldata ciphertext, bytes32 nonce) public view returns (bytes memory, bytes32, bytes memory, bytes32) {
        (uint256 oldSeq, AccountState[] memory thisCheckpoint) = abi.decode(Sapphire.decrypt(key, nonce, ciphertext, "checkpoint"), (uint256, AccountState[]));
        uint256 seqNum = oldSeq;
        uint256 nextSeqNum = seqNum + 1;
        require(seqNum < requests.length, "Sequence number too high. Send a request!");
        Request memory req = requests[seqNum];
        Response memory resp;
        if (req.rtype == RequestType.TRANSFER) {
            bool balanceOk = true;
            for (uint256 i = 0; i < thisCheckpoint.length; i++) {
                bool isSender = (thisCheckpoint[i].addr == req.from);
                balanceOk = balanceOk && (!isSender || thisCheckpoint[i].balance >= req.amount);
                thisCheckpoint[i].balance -= req.amount * (isSender ? 1 : 0) * (balanceOk ? 1 : 0);
            }
            bool foundRecipient = false;
            for (uint256 i = 0; i < thisCheckpoint.length; i++) {
                bool isRecipient = (thisCheckpoint[i].addr == req.to);
                foundRecipient = foundRecipient || isRecipient;
                thisCheckpoint[i].balance += req.amount * (isRecipient ? 1 : 0) * (balanceOk ? 1 : 0);
            }
            if (!foundRecipient) {
                // Add a new address whose initial value is the deposit amount
                AccountState[] memory newCheckpoint = new AccountState[](thisCheckpoint.length + 1);
                for (uint256 i = 0; i < thisCheckpoint.length; i++) {
                    newCheckpoint[i] = thisCheckpoint[i];
                }
                newCheckpoint[thisCheckpoint.length] = AccountState(req.to, req.amount);
                thisCheckpoint = newCheckpoint;
            }
            resp = Response(nextSeqNum, balanceOk, 0, balanceOk ? "transfer ok" : "not enough funds");
        } else if (req.rtype == RequestType.DEPOSIT) {
            bool found = false;
            for (uint256 i = 0; i < thisCheckpoint.length; i++) {
                bool isSender = (thisCheckpoint[i].addr == req.from);
                thisCheckpoint[i].balance += (req.amount) * (isSender ? 1 : 0);
                found = found || isSender;
            }
            // Leaks
            // Alternative: once it reaches 80% full, there is a random interaction that pads it with
            // empty addresses, and you'd write the address to one of the empty ones. A classic ORAM trick.
            if (!found) {
                // Add a new address whose initial value is the deposit amount
                AccountState[] memory newCheckpoint = new AccountState[](thisCheckpoint.length + 1);
                for (uint256 i = 0; i < thisCheckpoint.length; i++) {
                    newCheckpoint[i] = thisCheckpoint[i];
                }
                newCheckpoint[thisCheckpoint.length] = AccountState(req.from, req.amount);
                thisCheckpoint = newCheckpoint;
            }
            resp = Response(nextSeqNum, true, req.amount, "");
        } else if (req.rtype == RequestType.WITHDRAW) {
            bool balanceOk = true;
            for (uint256 i = 0; i < thisCheckpoint.length; i++) {
                bool isSender = (thisCheckpoint[i].addr == req.from);
                balanceOk = balanceOk && (!isSender || thisCheckpoint[i].balance >= req.amount);
                thisCheckpoint[i].balance -= req.amount * (isSender ? 1 : 0) * (balanceOk ? 1 : 0);
            }
            resp = Response(nextSeqNum, balanceOk, req.amount * (balanceOk ? 1 : 0), "");
        }
        
        bytes32 checkpointNonce = bytes32(Sapphire.randomBytes(32, ""));
        bytes memory newCheckpointCipher = Sapphire.encrypt(key, checkpointNonce, abi.encode(nextSeqNum, thisCheckpoint), "checkpoint");
        bytes32 responseNonce = bytes32(Sapphire.randomBytes(32, ""));
        bytes memory responseCipher = Sapphire.encrypt(key, responseNonce, abi.encode(resp), "response");
        return (newCheckpointCipher, checkpointNonce, responseCipher, responseNonce);
    }
    
    function commitResponse(bytes calldata ciphertext, bytes32 nonce) public {
        // This encryption handles authentication
        Response memory resp = abi.decode(Sapphire.decrypt(key, nonce, ciphertext, "response"), (Response));
        // Process strictly in order
        require(resp.seqNum == responses.length + 1, "Sequence number must be the next in order");
        responses.push(resp);
        
        Request memory request = requests[resp.seqNum - 1];
        if (request.rtype == RequestType.WITHDRAW) {
            address addr = request.from;
            payable(addr).transfer(resp.amount);
        }
        emit ResponseCommitted(resp.seqNum, resp);
    }
}
