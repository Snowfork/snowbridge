// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.20;

contract Agent {
    error Unauthorized();
    error InsufficientBalance();
    error WithdrawFailed();
    error InvokeFailed();

    // The unique ID for this agent, derived from the MultiLocation of the corresponding consensus system on Polkadot
    bytes32 public immutable agentID;

    // The gateway contract owning this agent
    address public immutable gateway;

    modifier onlyGateway() {
        if (msg.sender != gateway) {
            revert Unauthorized();
        }
        _;
    }

    constructor(bytes32 _agentID) {
        agentID = _agentID;
        gateway = msg.sender;
    }

    receive() external payable {}

    function withdrawTo(address payable recipient, uint256 amount) external onlyGateway {
        if (amount == 0) {
            return;
        }

        if (address(this).balance < amount) {
            revert InsufficientBalance();
        }

        (bool success,) = recipient.call{value: amount}("");
        if (!success) {
            revert WithdrawFailed();
        }
    }

    function invoke(address delegate, bytes calldata data) external onlyGateway returns (bytes memory) {
        (bool success, bytes memory result) = delegate.delegatecall(data);
        if (!success) {
            revert InvokeFailed();
        }

        return result;
    }
}
