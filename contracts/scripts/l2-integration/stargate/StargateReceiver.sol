// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

import {
    ILayerZeroComposer
} from "@layerzerolabs/lz-evm-protocol-v2/contracts/interfaces/ILayerZeroComposer.sol";
import {
    OFTComposeMsgCodec
} from "@layerzerolabs/lz-evm-oapp-v2/contracts/oft/libs/OFTComposeMsgCodec.sol";

contract StargateReceiver is ILayerZeroComposer {
    event ComposeAcknowledged(address indexed _from, uint256 _amount);

    address public endpoint;
    address public stargate;

    mapping(address => uint256) public acknowledgedCountByAddress;

    constructor(address _endpoint, address _stargate) {
        endpoint = _endpoint;
        stargate = _stargate;
    }

    function lzCompose(address _from, bytes32, bytes calldata _message, address, bytes calldata)
        external
        payable
    {
        require(_from == stargate, "!stargate");
        require(msg.sender == endpoint, "!endpoint");

        uint256 _amount = OFTComposeMsgCodec.amountLD(_message);
        bytes memory _composeMessage = OFTComposeMsgCodec.composeMsg(_message);

        // decode the compose message to get the original caller and do some logic based on that
        address caller = abi.decode(_composeMessage, (address));
        acknowledgedCountByAddress[caller]++;
        emit ComposeAcknowledged(caller, _amount);
    }

    fallback() external payable {}

    receive() external payable {}
}
