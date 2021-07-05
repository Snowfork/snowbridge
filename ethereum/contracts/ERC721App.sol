// SPDX-License-Identifier: MIT
pragma solidity ^0.8.5;
pragma experimental ABIEncoderV2;

import "@openzeppelin/contracts/access/AccessControl.sol";
import "@openzeppelin/contracts/token/ERC721/extensions/IERC721Metadata.sol";
import "./ScaleCodec.sol";
import "./OutboundChannel.sol";

enum ChannelId {Basic, Incentivized}

contract ERC721App is AccessControl {
    using ScaleCodec for uint256;

    mapping(ChannelId => Channel) public channels;

    bytes2 constant MINT_CALL = 0x4301;

    event Locked(
                 address tokenContract,
                 uint256 tokenId,
                 address sender,
                 bytes32 recipient
                 );

    event Unlocked(
                   address tokenContract,
                   uint256 tokenId,
                   bytes32 sender,
                   address recipient
                   );

    struct Channel {
        address inbound;
        address outbound;
    }

    bytes32 public constant INBOUND_CHANNEL_ROLE =
        keccak256("INBOUND_CHANNEL_ROLE");

    constructor(Channel memory _basic, Channel memory _incentivized) {
        Channel storage c1 = channels[ChannelId.Basic];
        c1.inbound = _basic.inbound;
        c1.outbound = _basic.outbound;

        Channel storage c2 = channels[ChannelId.Incentivized];
        c2.inbound = _incentivized.inbound;
        c2.outbound = _incentivized.outbound;

        _setupRole(INBOUND_CHANNEL_ROLE, _basic.inbound);
        _setupRole(INBOUND_CHANNEL_ROLE, _incentivized.inbound);
    }

    /**
     * @notice Locks ERC721 token in ERC721App contract
     * @param _tokenContract The NFT contract
     * @param _tokenId The NFT to lock
     * @param _recipient Polkadot address of the receiver
     * @param _channelId The Channel to use to send token
     */
    function lock(address _tokenContract,
                  uint256 _tokenId,
                  bytes32 _recipient,
                  ChannelId _channelId
                  ) public payable {
        require(
                _channelId == ChannelId.Basic ||
                _channelId == ChannelId.Incentivized,
                "Invalid channel ID"
                );

        IERC721Metadata token = IERC721Metadata(_tokenContract);

        require(token.ownerOf(_tokenId) == msg.sender, "Transfer of token that is not own");

        token.transferFrom(msg.sender, address(this), _tokenId);

        emit Locked(_tokenContract, _tokenId, msg.sender, _recipient);

        bytes memory call = encodeCall(_tokenContract, _tokenId, msg.sender, _recipient, token.tokenURI(_tokenId));

        OutboundChannel channel =
            OutboundChannel(channels[_channelId].outbound);
        channel.submit(msg.sender, call);
    }

    /**
     * @notice Unlocks ERC721 token from ERC721App contract
     * @param _tokenContract The NFT contract
     * @param _tokenId The NFT to lock
     * @param _sender Polkadot address of the sender
     * @param _recipient The ETHApp
     */
    function unlock(
             address _tokenContract,
             uint256 _tokenId,
             bytes32 _sender,
             address _recipient
             ) public {
        require(
            hasRole(INBOUND_CHANNEL_ROLE, msg.sender),
            "Caller is not an inbound channel"
        );

        IERC721Metadata token = IERC721Metadata(_tokenContract);

        require(token.ownerOf(_tokenId) == address(this), "Transfer of token that is not own");

        token.transferFrom(address(this), _recipient, _tokenId);
        emit Unlocked(_tokenContract, _tokenId, _sender, _recipient);
    }

    // SCALE-encode payload
    function encodeCall(
                        address _tokenContract,
                        uint256 _tokenId,
                        address _sender,
                        bytes32 _recipient,
                        string memory _tokenURI
                        ) private pure returns (bytes memory) {
        return
            abi.encodePacked(
                             MINT_CALL,
                             _sender,
                             bytes1(0x00), // Encode recipient as MultiAddress::Id
                             _recipient,
                             _tokenContract,
                             _tokenId.encode256(),
                             bytes1(0x00) // TODO placeholder for the following, which requires compact (scale) encoded length prefix: // bytes(_tokenURI)
                            );
    }
}

