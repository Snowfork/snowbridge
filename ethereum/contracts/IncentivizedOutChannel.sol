 // SPDX-License-Identifier: MIT
pragma solidity >=0.6.2;

import "./OutChannel.sol";

// IncentivizedOutChannel is a channel that outputs ordered messages with an increasing nonce. It will have incentivization too.
contract IncentivizedOutChannel is OutChannel  {

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
