// SPDX-License-Identifier: MIT
pragma solidity >=0.6.2;

import "@openzeppelin/contracts/math/SafeMath.sol";
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";

contract ERC20App {
    using SafeMath for uint256;

    uint256 public nonce;
    mapping(address => uint256) public totalTokens;

    event AppEvent(bytes _data);
    event Unlock(address _recipient, address _token, uint256 _amount);

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
        emit AppEvent(data);
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

    function submit(bytes memory data)
        public
    {
        // TODO: decode message bytes into (tag, recipient, token, amount)
         uint256 tag = 0;                        // placeholder
         address payable recipient = address(0); // placeholder
         address token = address(0);             // placeholder
         uint256 amount = 1;                     // placeholder

         unlockERC20(recipient, token, amount);
    }

    function unlockERC20(
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
        emit Unlock(_recipient, _token, _amount);
    }
}
