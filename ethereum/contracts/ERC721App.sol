// SPDX-License-Identifier: MIT
pragma solidity >=0.7.6;
pragma experimental ABIEncoderV2;

import "@openzeppelin/contracts/token/ERC721/IERC721Metadata.sol";
import "./ScaleCodec.sol";
import "./OutboundChannel.sol";

enum ChannelId {Basic, Incentivized}

contract ERC721App {
    using ScaleCodec for uint256;

    mapping(ChannelId => Channel) public channels;

    bytes2 constant MINT_CALL = 0x0d01;

    event Locked(
                 address token,
                 address sender,
                 bytes32 recipient,
                 uint256 tokenId
                 );

    event Unlocked(
                   address token,
                   bytes32 sender,
                   address recipient,
                   uint256 tokenId
                   );

    struct Channel {
        address inbound;
        address outbound;
    }

    constructor(Channel memory _basic, Channel memory _incentivized) {
        Channel storage c1 = channels[ChannelId.Basic];
        c1.inbound = _basic.inbound;
        c1.outbound = _basic.outbound;

        Channel storage c2 = channels[ChannelId.Incentivized];
        c2.inbound = _incentivized.inbound;
        c2.outbound = _incentivized.outbound;
    }

    /**
     * @notice Locks ERC721 token in ERC721App contract
     * @param _token The NFT contract
     * @param _recipient Polkadot address of the receiver
     * @param _tokenId The NFT to lock
     * @param _channelId The Channel to use to send token
     */
    function lock(address _token,
                  bytes32 _recipient,
                  uint256 _tokenId,
                  ChannelId _channelId
                  ) public payable {
        require(
                _channelId == ChannelId.Basic ||
                _channelId == ChannelId.Incentivized,
                "Invalid channel ID"
                );

        IERC721Metadata token = IERC721Metadata(_token);

        require(token.ownerOf(_tokenId) == msg.sender, "Transfer of token that is not own");

        token.transferFrom(msg.sender, address(this), _tokenId);

        emit Locked(_token, msg.sender, _recipient, _tokenId);

        bytes memory call = encodeCall(msg.sender, _recipient, _token, _tokenId, token.tokenURI(_tokenId));

        OutboundChannel channel =
            OutboundChannel(channels[_channelId].outbound);
        channel.submit(msg.sender, call);

    }

    /**
     * @notice Unlocks ERC721 token from ERC721App contract
     * @param _token The NFT contract
     * @param _sender Polkadot address of the sender
     * @param _recipient The ETHApp
     * @param _tokenId The NFT to lock
     */
    function unlock(
             address _token,
             bytes32 _sender,
             address _recipient,
             uint256 _tokenId
             ) public {
        // TODO: Ensure message sender is a known inbound channel
        IERC721Metadata token = IERC721Metadata(_token);

        require(token.ownerOf(_tokenId) == address(this), "Transfer of token that is not own");

        token.transferFrom(address(this), _recipient, _tokenId);
        emit Unlocked(_token, _sender, _recipient, _tokenId);
    }

    // SCALE-encode payload
    function encodeCall(
                        address _sender,
                        bytes32 _recipient,
                        address _token,
                        uint256 _tokenId,
                        string memory _tokenURI
                        ) private pure returns (bytes memory) {
        return
            abi.encodePacked(
                             MINT_CALL,
                             _sender,
                             byte(0x00), // Encode recipient as MultiAddress::Id
                             _recipient,
                             _token,
                             _tokenId.encode256(),
                             bytes(_tokenURI)
                             );
    }
}

