// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

import {Script} from "forge-std/Script.sol";

import {StargateAdaptor} from "./StargateAdaptor.sol";
import {StargateComposer} from "./StargateComposer.sol";
import {SEPOLIA_STARGATE, OPT_CHAIN_EID} from "./Constants.sol";

// sent eth from source to destination
contract TransferEthFromAdaptor is Script {
    StargateComposer public stargateComposer;
    StargateAdaptor public stargateAdaptor;

    uint256 internal deployerPrivateKey = vm.envUint("DEPLOYER_KEY");
    address deployerAddr = vm.addr(deployerPrivateKey);

    function setUp() public {}

    function run() public {
        vm.startBroadcast(deployerPrivateKey);

        // ! fill with the deployed composer addresses
        stargateComposer = StargateComposer(vm.envAddress("COMPOSER_ADDRESS"));

        // ! fill with the deployed adaptor addresses
        stargateAdaptor = StargateAdaptor(payable(vm.envAddress("ADAPTOR_ADDRESS")));

        uint256 amountToSend = 0.002 ether;

        bytes memory _composeMsg = abi.encode(deployerAddr);

        (uint256 valueToSend,,) = stargateComposer.prepare(
            SEPOLIA_STARGATE, // stargate
            OPT_CHAIN_EID, // destinationEndpointId
            amountToSend, // amount
            vm.envAddress("RECEIVER_ADDRESS"), // to
            _composeMsg, // composeMsg
            200_000 // composeFunctionGasLimit
        );

        // Pre-fund the adaptor
        payable(address(stargateAdaptor)).transfer(valueToSend);

        stargateAdaptor.sendToken(amountToSend);

        vm.stopBroadcast();
    }
}
