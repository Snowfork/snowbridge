// SPDX-License-Identifier: MIT
pragma solidity >=0.6.2;

import "@openzeppelin/contracts/math/SafeMath.sol";
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "./Decoder.sol";
import "./Application.sol";

contract ERC20App is Application {
    using SafeMath for uint256;
    using Decoder for bytes;

    uint32 MESSAGE_LENGTH = 104;

    uint256 public nonce;
    mapping(address => uint256) public totalTokens;

    enum AppEventTags { SendETH, SendERC20 }

    event AppEvent(uint _tag, bytes _data);
    event Unlock(bytes _sender, address _recipient, address _token, uint256 _amount);

    constructor() public {
        nonce = 0;
    }

    function sendERC20(
        bytes32 _recipient,
        address _tokenAddr,
        uint256 _amount
    )
        public
    {
       require(
            IERC20(_tokenAddr).transferFrom(msg.sender, address(this), _amount),
            "Contract token allowances insufficient to complete this lock request"
        );

        // Increment locked ERC20 token counter by this amount
        totalTokens[_tokenAddr] = totalTokens[_tokenAddr].add(_amount);
        // Increment global nonce
        nonce = nonce.add(1);

        bytes memory data = encodeSendData(msg.sender, _recipient, _tokenAddr,_amount, nonce);
        emit AppEvent(uint(AppEventTags.SendERC20), data);
    }

    function encodeSendData(
        address _sender,
        bytes32 _recipient,
        address _tokenAddr,
        uint256 _amount,
        uint256 _nonce
    )
        internal
        pure
        returns(bytes memory)
    {
        return abi.encode(_sender, _recipient, _tokenAddr, _amount, _nonce);
    }

    function handle(bytes memory _message)
        public
        override
    {
        require(_message.length == MESSAGE_LENGTH, "Message must contain 104 bytes for a successful decoding");

        // Decode sender bytes
        bytes memory sender = _message.slice(0, 32);
        // Decode recipient address
        address recipient = _message.sliceAddress(32);
        // Decode token address
        address tokenAddr = _message.sliceAddress(32 + 20);
        // Deocde amount int256
        bytes memory amountBytes = _message.slice(32 + 40, 32);
        uint256 amount = amountBytes.decodeUint256();

        sendTokens(recipient, tokenAddr, amount);
        emit Unlock(sender, recipient, tokenAddr, amount);
    }

    function sendTokens(
        address _recipient,
        address _token,
        uint256 _amount
    )
        internal
    {
        require(_amount > 0, "Must unlock a positive amount");
        require(totalTokens[_token] > _amount, "ERC20 token balances insufficient to fulfill the unlock request");

        totalTokens[_token] = totalTokens[_token].sub(_amount);
        require(IERC20(_token).transfer(_recipient, _amount), "ERC20 token transfer failed");
    }
}
