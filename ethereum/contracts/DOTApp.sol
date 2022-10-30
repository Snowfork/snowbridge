// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

import "@openzeppelin/contracts/access/AccessControl.sol";
import "./WrappedToken.sol";
import "./OutboundChannel.sol";
import "./FeeController.sol";
import "./DOTAppPallet.sol";
import "./ChannelRegistry.sol";

contract DOTApp is FeeController, AccessControl {
    WrappedToken public immutable token;
    ChannelRegistry public immutable registry;

    bytes32 public constant FEE_BURNER_ROLE = keccak256("FEE_BURNER_ROLE");

    event Minted(bytes32 sender, address recipient, uint256 amount);
    event Burned(address sender, bytes32 recipient, uint256 amount);

    error UnknownChannel(uint32 id);
    error Unauthorized();

    constructor(
        WrappedToken _token,
        address feeBurner,
        address channelRegistry
    ) {
        token = _token;
        registry = ChannelRegistry(channelRegistry);
        _setupRole(FEE_BURNER_ROLE, feeBurner);
    }

    function burn(
        bytes32 _recipient,
        uint256 _amount,
        uint32 _channelID
    ) external {
        token.burn(msg.sender, _amount);

        address channel = registry.outboundChannelForID(_channelID);
        if (channel == address(0)) {
            revert UnknownChannel(_channelID);
        }

        (bytes memory call, uint64 weight) = DOTAppPallet.unlock(msg.sender, _recipient, _amount);
        OutboundChannel(channel).submit(msg.sender, call, weight);

        emit Burned(msg.sender, _recipient, _amount);
    }

    function mint(
        bytes32 _sender,
        address _recipient,
        uint256 _amount
    ) external {
        if (!registry.isInboundChannel(msg.sender)) {
            revert Unauthorized();
        }
        token.mint(_recipient, _amount);
        emit Minted(_sender, _recipient, _amount);
    }

    // Incentivized channel calls this to charge (burn) fees
    function handleFee(address feePayer, uint256 _amount)
        external
        override
        onlyRole(FEE_BURNER_ROLE)
    {
        token.burn(feePayer, _amount);
    }
}
