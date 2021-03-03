// SPDX-License-Identifier: MIT
pragma solidity >=0.7.6;
pragma experimental ABIEncoderV2;

import "./WrappedToken.sol";
import "./ScaleCodec.sol";
import "./OutboundChannel.sol";

enum ChannelId {Basic, Incentivized}

abstract contract BaseDOTApp {
    using ScaleCodec for uint128;

    mapping(ChannelId => Channel) public channels;

    bytes2 constant UNLOCK_CALL = 0x0e01;

    WrappedToken public token;

    struct Channel {
        address inbound;
        address outbound;
    }

    constructor(
        string memory _name,
        string memory _symbol,
        Channel memory _basic,
        Channel memory _incentivized
    ) {
        address[] memory defaultOperators;
        token = new WrappedToken(_name, _symbol, defaultOperators);

        Channel storage c1 = channels[ChannelId.Basic];
        c1.inbound = _basic.inbound;
        c1.outbound = _basic.outbound;

        Channel storage c2 = channels[ChannelId.Incentivized];
        c2.inbound = _incentivized.inbound;
        c2.outbound = _incentivized.outbound;
    }

    function burn(bytes32 _recipient, uint256 _amount, ChannelId _channelId) public {
        require(
            _channelId == ChannelId.Basic ||
            _channelId == ChannelId.Incentivized,
            "Invalid channel ID"
        );
        require(_amount % granularity() == 0, "Invalid Granularity");

        token.burn(msg.sender, _amount, abi.encodePacked(_recipient));

        OutboundChannel channel = OutboundChannel(channels[_channelId].outbound);

        bytes memory call = encodeCall(msg.sender, _recipient, unwrap(_amount));
        channel.submit(call);
    }

    function burnFee(address _user, uint256 _amount) external {
        // TODO: Ensure message sender is a known outbound channel
        token.burn(_user, _amount, "");
    }

    function mint(bytes32 _sender, address _recipient, uint128 _amount) public {
        // TODO: Ensure message sender is a known inbound channel
        token.mint(_recipient, wrap(_amount), abi.encodePacked(_sender));
    }

    function encodeCall(address _sender, bytes32 _recipient, uint128 _amount)
        private
        pure
        returns (bytes memory)
    {
        return
            abi.encodePacked(
                UNLOCK_CALL,
                _sender,
                byte(0x00), // Encoding recipient as MultiAddress::Id
                _recipient,
                _amount.encode128()
            );
    }

    /*
     * Convert native DOT/KSM/ROC to the wrapped equivalent.
     *
     * SAFETY: No need for SafeMath.mul since its impossible to overflow
     * when 0 <= granularity <= 10 ^ 8, as specified by DOTAppDecimals10.sol
     * and DOTAppDecimals12.sol.
     *
     * Can verify in Rust using this snippet:
     *
     *   let granularity = U256::from(100000000u64);
     *   U256::from(u128::MAX).checked_mul(granularity).unwrap();
     *
     */
    function wrap(uint128 _value) pure internal returns (uint256) {
        return uint256(_value) * granularity();
    }

    /*
     * Convert wrapped DOT/KSM/ROC to its native equivalent.
     *
     * SAFETY: No need for SafeMath.div since granularity() resolves to a non-zero
     * constant (See DOTAppDecimals10.sol and DOTAppDecimals12.sol)
     */
    function unwrap(uint256 _value) pure internal returns (uint128) {
        return uint128(_value / granularity());
    }

    /**
     * Smallest part of DOT/KSM/ROC that is not divisible when increasing
     * precision to 18 decimal places.
     *
     * This is used for converting between native and wrapped
     * representations of DOT/KSM/ROC.
    */
    function granularity() pure internal virtual returns (uint256);

}
