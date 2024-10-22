// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.25;

import {IGateway} from "./interfaces/IGateway.sol";
import {WETH9} from "canonical-weth/WETH9.sol";
import {ParaID, MultiAddress} from "./Types.sol";

contract DepositWETH9AndSend {
    address public immutable GATEWAY_PROXY;
    address payable public immutable WETH_INSTANCE;

    constructor(address _gatewayProxy, address payable _wethInstance) {
        GATEWAY_PROXY = _gatewayProxy;
        WETH_INSTANCE = _wethInstance;
    }

    // Transfer ERC20 tokens to a Polkadot parachain
    function sendToken(
        ParaID destinationChain,
        MultiAddress calldata destinationAddress,
        uint128 destinationFee,
        uint128 amount
    ) external payable {
        WETH9(WETH_INSTANCE).deposit{value: amount}();
        WETH9(WETH_INSTANCE).approve(GATEWAY_PROXY, amount);

        uint256 fee = msg.value - amount;
        IGateway(GATEWAY_PROXY).sendToken{value: fee}(
            WETH_INSTANCE, destinationChain, destinationAddress, destinationFee, amount
        );
    }
}
