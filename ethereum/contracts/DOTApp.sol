// SPDX-License-Identifier: MIT
pragma solidity >=0.7.6;
pragma experimental ABIEncoderV2;

import "@openzeppelin/contracts/access/AccessControl.sol";
import "./WrappedToken.sol";
import "./ScaleCodec.sol";
import "./OutboundChannel.sol";

enum ChannelId {Basic, Incentivized}

contract DOTApp is AccessControl {
    using ScaleCodec for uint128;

    bytes32 public constant FEE_BURNER_ROLE = keccak256("FEE_BURNER_ROLE");

    mapping(ChannelId => Channel) public channels;

    bytes2 constant UNLOCK_CALL = 0x0e01;

    /*
     * Smallest part of DOT/KSM/ROC that is not divisible when increasing
     * precision to 18 decimal places.
     *
     * This is used for converting between native and wrapped
     * representations of DOT/KSM/ROC.
    */
    uint256 private granularity;

    WrappedToken public token;

    struct Channel {
        address inbound;
        address outbound;
    }

    constructor(
        string memory _name,
        string memory _symbol,
        uint256 _decimals,
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

        _setupRole(FEE_BURNER_ROLE, _incentivized.outbound);

        granularity = 10 ** (18 - _decimals);
    }

    function burn(bytes32 _recipient, uint256 _amount, ChannelId _channelId) public {
        require(
            _channelId == ChannelId.Basic ||
            _channelId == ChannelId.Incentivized,
            "Invalid channel ID"
        );
        require(_amount % granularity == 0, "Invalid Granularity");

        token.burn(msg.sender, _amount, abi.encodePacked(_recipient));

        OutboundChannel channel = OutboundChannel(channels[_channelId].outbound);

        bytes memory call = encodeCall(msg.sender, _recipient, unwrap(_amount));
        channel.submit(msg.sender, call);
    }

    function mint(bytes32 _sender, address _recipient, uint128 _amount) public {
        // TODO: Ensure message sender is a known inbound channel
        token.mint(_recipient, wrap(_amount), abi.encodePacked(_sender));
    }

    function burnFee(address _account, uint256 _amount) external returns (uint128) {
        require(hasRole(FEE_BURNER_ROLE, msg.sender), "ACCESS_FORBIDDEN");
        require(_amount % granularity == 0, "INVALID_GRANULARITY");
        token.burn(_account, _amount, "");
        return unwrap(_amount);
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
     * SAFETY: No need for SafeMath.mul as overflow is not possible for
     * 0 <= granularity <= 10 ^ 8.
     *
     * Can verify in Rust using this snippet:
     *
     *   let granularity = U256::from(100000000u64);
     *   U256::from(u128::MAX).checked_mul(granularity).unwrap();
     *
     */
    function wrap(uint128 _value) view internal returns (uint256) {
        return uint256(_value) * granularity;
    }

    /*
     * Convert wrapped DOT/KSM/ROC to its native equivalent.
     *
     * SAFETY: No need for SafeMath.div since granularity is
     * configured to be non-zero.
     */
    function unwrap(uint256 _value) view internal returns (uint128) {
        return uint128(_value / granularity);
    }
}
