// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

import {Script, console} from "forge-std/Script.sol";
import {IERC20} from "openzeppelin/token/ERC20/IERC20.sol";
import {SnowbridgeL2Adaptor} from "./SnowbridgeL2Adaptor.sol";
import {USDC, BASE_USDC, CHAIN_ID, BASE_CHAIN_ID, BASE_WETH9} from "./Constants.sol";
import {ISpokePool, IMessageHandler} from "./Interfaces.sol";
import {SwapParams, SendParams} from "./Types.sol";
import {WETH9} from "canonical-weth/WETH9.sol";

contract TestSnowbridgeL2Adaptor is Script {
    uint256 internal deployerPrivateKey = vm.envUint("DEPLOYER_KEY");
    address deployerAddr = vm.addr(deployerPrivateKey);

    function setUp() public {}

    function run() public {
        vm.startBroadcast(deployerPrivateKey);

        address payable l2SnowbridgeAdaptor =
            payable(vm.envAddress("L2_SNOWBRIDGE_ADAPTOR_ADDRESS"));
        SwapParams memory params = SwapParams({
            inputToken: BASE_USDC,
            outputToken: USDC,
            inputAmount: 110_000, // 0.11 USDC
            outputAmount: 100_000, // 0.1 USDC
            destinationChainId: CHAIN_ID
        });
        //Parameter from https://sepolia.etherscan.io/tx/0xe666671edd1559666990d685b53d5982b2b752444a3652d7f90fc5a6c33b2c09
        SendParams memory sendParams = SendParams({
            xcm: hex"050c140d010208000101005827013ddc4082f8252f8729bd2f06e77e7863dea9202a6f0e7a2c34e356e85a2c6f501789818b10452a5835054d3fd74c288d611ec97f194a997bef41f68fdb40",
            assets: new bytes[](0),
            claimer: hex"000101005827013ddc4082f8252f8729bd2f06e77e7863dea9202a6f0e7a2c34e356e85a",
            executionFee: 33_258_626_953_769, // 0.000033258626953769 ETH
            relayerFee: 552_354_417_681_961, // 0.000552354417681961 ETH
            l2Fee: 10_000_000_000_000
        });

        uint256 nativeFeeAmount =
            sendParams.relayerFee + sendParams.executionFee + sendParams.l2Fee;

        IERC20(params.inputToken).transfer(l2SnowbridgeAdaptor, params.inputAmount);
        payable(l2SnowbridgeAdaptor).transfer(nativeFeeAmount);

        SnowbridgeL2Adaptor(l2SnowbridgeAdaptor).swapTokenAndCall(params, sendParams, deployerAddr);

        vm.stopBroadcast();
        return;
    }
}
