// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

import {IStargate} from "@stargatefinance/stg-evm-v2/src/interfaces/IStargate.sol";

import {
    MessagingFee,
    SendParam
} from "@layerzerolabs/lz-evm-oapp-v2/contracts/oft/interfaces/IOFT.sol";

import {Script} from "forge-std/Script.sol";

import {StargateReceiver} from "./StargateReceiver.sol";
import {StargateComposer} from "./StargateComposer.sol";

import {SEPOLIA_STARGATE, OPT_CHAIN_EID} from "./Constants.sol";

// sent eth from source to destination
contract TransferEth is Script {
    StargateComposer public stargateComposer;
    StargateReceiver public stargateReceiver;

    uint256 internal deployerPrivateKey = vm.envUint("DEPLOYER_KEY");
    address deployerAddr = vm.addr(deployerPrivateKey);

    function setUp() public {}

    function run() public {
        vm.startBroadcast(deployerPrivateKey);

        // ! fill with the deployed composer addresses
        stargateComposer = StargateComposer(vm.envAddress("COMPOSER_ADDRESS"));
        // ! fill with the deployed receiver addresses
        stargateReceiver = StargateReceiver(payable(vm.envAddress("RECEIVER_ADDRESS")));

        bytes memory _composeMsg = abi.encode(deployerAddr);

        (uint256 valueToSend, SendParam memory sendParam, MessagingFee memory messagingFee) = stargateComposer.prepare(
            SEPOLIA_STARGATE, // stargate
            OPT_CHAIN_EID, // destinationEndpointId
            0.002 ether, // amount
            address(stargateReceiver), // to
            _composeMsg, // composeMsg
            200_000 // composeFunctionGasLimit
        );

        IStargate(SEPOLIA_STARGATE)
        .sendToken{value: valueToSend}(sendParam, messagingFee, deployerAddr);

        vm.stopBroadcast();
    }
}
