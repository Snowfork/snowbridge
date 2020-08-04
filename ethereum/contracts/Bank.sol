// SPDX-License-Identifier: MIT
pragma solidity ^0.6.2;

import "@openzeppelin/contracts/math/SafeMath.sol";
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";

contract Bank {
    using SafeMath for uint256;

    uint256 public nonce;
    uint256 public totalETH;
    mapping(address => uint256) public totalTokens;


    enum AppEventTags { SendETH, SendERC20 }

    event AppEvent(uint _tag, bytes _data);

    constructor() public {
        nonce = 0;
    }

    function sendETH(
        bytes32 _recipient
    )
        public payable
    {
        require(msg.value > 0, "Value of transaction must be positive");

        // Increment locked Ethereum counter by this amount
        totalETH = totalETH.add(msg.value);
        // Increment global nonce
        nonce = nonce.add(1);

        bytes memory data = encodeSendData(msg.sender, _recipient, address(0), msg.value, nonce);
        emit AppEvent(uint(AppEventTags.SendETH), data);
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
}
