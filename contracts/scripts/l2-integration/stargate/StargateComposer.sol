// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

import {IStargate} from "@stargatefinance/stg-evm-v2/src/interfaces/IStargate.sol";
import {
    MessagingFee,
    OFTReceipt,
    SendParam
} from "@layerzerolabs/lz-evm-oapp-v2/contracts/oft/interfaces/IOFT.sol";
import {
    OptionsBuilder
} from "@layerzerolabs/lz-evm-oapp-v2/contracts/oapp/libs/OptionsBuilder.sol";

contract StargateComposer {
    using OptionsBuilder for bytes;

    function prepare(
        address _stargate,
        uint32 _dstEid,
        uint256 _amount,
        address _composer,
        bytes memory _composeMsg, // abi encoded elements for the compose call
        uint128 _composeFunctionGasLimit
    )
        external
        view
        returns (uint256 valueToSend, SendParam memory sendParam, MessagingFee memory messagingFee)
    {
        bytes memory extraOptions = _composeMsg.length > 0
            ? OptionsBuilder.newOptions()
                .addExecutorLzComposeOption(
                    0, // compose call function index
                    _composeFunctionGasLimit, // compose function gas limit
                    0 // compose function msg value
                )  // compose gas limit
            : bytes("");

        sendParam = SendParam({
            dstEid: _dstEid,
            to: addressToBytes32(_composer),
            amountLD: _amount, // amount to send
            minAmountLD: _amount,
            extraOptions: extraOptions,
            composeMsg: _composeMsg,
            oftCmd: ""
        });

        IStargate stargate = IStargate(_stargate);

        (,, OFTReceipt memory receipt) = stargate.quoteOFT(sendParam);
        sendParam.minAmountLD = receipt.amountReceivedLD;

        messagingFee = stargate.quoteSend(sendParam, false); // get the fee (stgFee and lzFee)
        valueToSend = messagingFee.nativeFee; // add the stargate fee to the value

        if (stargate.token() == address(0x0)) {
            valueToSend += sendParam.amountLD; // add the amount to send to the value
        }
    }

    function addressToBytes32(address _addr) internal pure returns (bytes32) {
        return bytes32(uint256(uint160(_addr)));
    }
}
