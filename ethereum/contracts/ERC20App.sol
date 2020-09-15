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

    event Transfer(address _sender, bytes32 _recipient, address _token, uint256 _amount);
    event Unlock(bytes _sender, address _recipient, address _token, uint256 _amount);

    constructor() public {
        nonce = 0;
    }

    function sendERC20(bytes32 _recipient, address _tokenAddr, uint256 _amount)
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

        emit Transfer(msg.sender, _recipient, _tokenAddr, _amount);
    }

    function handle(bytes memory _data)
        public
        override
    {
        require(_data.length == MESSAGE_LENGTH, "Message must contain 104 bytes for a successful decoding");

        // Decode sender bytes
        bytes memory sender = _data.slice(0, 32);
        // Decode recipient address
        address recipient = _data.sliceAddress(32);
        // Decode token address
        address tokenAddr = _data.sliceAddress(32 + 20);
        // Deocde amount int256
        bytes memory amountBytes = _data.slice(32 + 40, 32);
        uint256 amount = amountBytes.decodeUint256();

        sendTokens(recipient, tokenAddr, amount);
        emit Unlock(sender, recipient, tokenAddr, amount);
    }

    function sendTokens(address _recipient, address _token, uint256 _amount)
        internal
    {
        require(_amount > 0, "Must unlock a positive amount");
        require(totalTokens[_token] > _amount, "ERC20 token balances insufficient to fulfill the unlock request");

        totalTokens[_token] = totalTokens[_token].sub(_amount);
        require(IERC20(_token).transfer(_recipient, _amount), "ERC20 token transfer failed");
    }
}
