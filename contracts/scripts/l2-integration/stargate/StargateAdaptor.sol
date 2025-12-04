// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

import {IStargate} from "@stargatefinance/stg-evm-v2/src/interfaces/IStargate.sol";
import {
    MessagingFee,
    SendParam
} from "@layerzerolabs/lz-evm-oapp-v2/contracts/oft/interfaces/IOFT.sol";

import {StargateComposer} from "./StargateComposer.sol";
import {StargateReceiver} from "./StargateReceiver.sol";
import {SEPOLIA_STARGATE, OPT_CHAIN_EID} from "./Constants.sol";

contract StargateAdaptor {
    StargateComposer public composer;
    StargateReceiver public receiver;
    uint32 internal constant DEST_EID = OPT_CHAIN_EID; // optimism-sepolia
    address constant STARGATE = SEPOLIA_STARGATE; // sepolia stargate address

    constructor(address _composer, address _receiver) {
        composer = StargateComposer(_composer);
        receiver = StargateReceiver(payable(_receiver));
    }

    function sendToken(uint256 amount) external payable {
        // For ERC20 token transfer, approve the stargate contract to spend tokens first
        // ERC20(token).approve(stargate, amount);
        bytes memory composeMsg = abi.encode(msg.sender);

        (uint256 valueToSend, SendParam memory sendParam, MessagingFee memory messagingFee) = composer.prepare(
            STARGATE, // stargate
            DEST_EID, // destinationEndpointId
            amount, // amount
            address(receiver), // to
            composeMsg, // composeMsg
            200_000 // composeFunctionGasLimit
        );

        IStargate(STARGATE).sendToken{value: valueToSend}(sendParam, messagingFee, msg.sender);
    }

    receive() external payable {}
}
