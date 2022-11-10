// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

import "@openzeppelin/contracts/access/AccessControl.sol";
import "./OutboundChannel.sol";
import "./ERC20AppPallet.sol";
import "./ERC20Vault.sol";
import "./ChannelRegistry.sol";

contract ERC20App is AccessControl {
    using SafeERC20 for IERC20;

    ERC20Vault public immutable vault;
    ChannelRegistry public immutable registry;

    // Registered tokens
    mapping(address => bool) public tokens;

    // Unknown outbound channel
    error UnknownChannel(uint32 id);

    // Not allowed to send messages to this app
    error Unauthorized();

    // ERC20 transfer amount should be greater than zero
    error MinimumAmount();

    event Locked(
        address token,
        address sender,
        bytes32 recipient,
        uint128 amount,
        uint32 paraId,
        uint128 fee
    );

    event Unlocked(
        address token,
        bytes32 sender,
        address recipient,
        uint128 amount
    );

    constructor(
        ERC20Vault erc20vault,
        ChannelRegistry channelRegistry
    ) {
        vault = erc20vault;
        registry = channelRegistry;
    }

    function lock(
        address _token,
        bytes32 _recipient,
        uint128 _amount,
        uint32 _paraID,
        uint128 _fee,
        uint32 _channelID
    ) public {
        address channel = registry.outboundChannelForID(_channelID);
        if (channel == address(0)) {
            revert UnknownChannel(_channelID);
        }

        vault.deposit(msg.sender, _token, _amount);

        if (!tokens[_token]) {
            (bytes memory createCall, uint64 createWeight) = ERC20AppPallet.create(_token);
            tokens[_token] = true;
            OutboundChannel(channel).submit(msg.sender, createCall, createWeight);
        }

        bytes memory call;
        uint64 weight;
        if (_paraID == 0) {
            (call, weight) = ERC20AppPallet.mint(_token, msg.sender, _recipient, _amount);
        } else {
            (call, weight) = ERC20AppPallet.mintAndForward(_token, msg.sender, _recipient, _amount, _paraID, _fee);
        }
        OutboundChannel(channel).submit(msg.sender, call, weight);

        emit Locked(_token, msg.sender, _recipient, _amount, _paraID, _fee);
    }

    function unlock(
        address _token,
        bytes32 _sender,
        address _recipient,
        uint128 _amount
    ) external {
        if (!registry.isInboundChannel(msg.sender)) {
            revert Unauthorized();
        }

        if (_amount == 0) {
            revert MinimumAmount();
        }

        vault.withdraw(_recipient, _token, _amount);
        emit Unlocked(_token, _sender, _recipient, _amount);
    }

    function transferVaultOwnership(
        address newOwner
    ) external {
        if (!registry.isInboundChannel(msg.sender)) {
            revert Unauthorized();
        }
        vault.transferOwnership(newOwner);
    }
}
