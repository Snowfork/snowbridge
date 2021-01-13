 // SPDX-License-Identifier: MIT
pragma solidity >=0.6.2;

import "./OutChannel.sol";

// BasicOutChannel is a basic channel that just outputs messages with a nonce.
contract BasicOutChannel is OutChannel  {

    uint256 public currentNonce;

    event NewMessage(uint256 nonce, address senderAddress, string targetApplicationId, bytes payload);

    constructor() public {
        currentNonce = 0;
    }

    /**
     * @dev Submits a message to the channel
     */
    function submit(string memory targetApplicationId, bytes memory payload) public override {
        emit NewMessage(currentNonce, msg.sender, targetApplicationId, payload);
        currentNonce++;
    }

}
