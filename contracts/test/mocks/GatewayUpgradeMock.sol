// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.20;

import {Channel, InboundMessage, OperatingMode, ParaID, Config, Command} from "../../src/Types.sol";
import {IGateway} from "../../src/interfaces/IGateway.sol";
import {IInitializable} from "../../src/interfaces/IInitializable.sol";
import {Verification} from "../../src/Verification.sol";

contract GatewayUpgradeMock is IGateway, IInitializable {
    /**
     * Getters
     */

    function operatingMode() external pure returns (OperatingMode) {
        return OperatingMode.Normal;
    }

    function channelOperatingModeOf(ParaID) external pure returns (OperatingMode) {
        return OperatingMode.Normal;
    }

    function channelFeeOf(ParaID) external pure returns (uint256) {
        return 0;
    }

    function channelNoncesOf(ParaID) external pure returns (uint64, uint64) {
        return (0, 0);
    }

    function agentOf(bytes32) external pure returns (address) {
        return address(0);
    }

    function tokenTransferFees() external pure returns (uint256, uint256) {
        return (1, 1);
    }

    function implementation() external pure returns (address) {
        return address(0);
    }

    function submitInbound(InboundMessage calldata, bytes32[] calldata, Verification.Proof calldata) external {}

    function registerToken(address) external payable {}
    function sendToken(address, ParaID, bytes32, uint128) external payable {}
    function sendToken(address, ParaID, address, uint128) external payable {}

    event Initialized(uint256 d0, uint256 d1);

    function initialize(bytes memory data) external {
        // Just decode and exit
        (uint256 d0, uint256 d1) = abi.decode(data, (uint256, uint256));
        emit Initialized(d0, d1);
    }
}
