// SPDX-License-Identifier: MIT
pragma solidity >=0.6.2;

import "@openzeppelin/contracts/math/SafeMath.sol";

contract EthereumApp {
    using SafeMath for uint256;

    uint256 public nonce;
    uint256 public totalETH;

    event AppEvent(bytes _data);
    event Unlock(address _recipient, uint256 _amount);

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
        // TODO: decode message bytes into (tag, recipient, amount)
         uint256 tag = 0;                        // placeholder
         address payable recipient = address(0); // placeholder
         uint256 amount = 1;                     // placeholder

        unlockETH(recipient, amount);
    }

    function unlockETH(
        address payable _recipient,
        uint256 _amount
    )
        internal
    {
        require(_amount > 0, "Must unlock a positive amount");
        require(totalETH > _amount, "ETH token balances insufficient to fulfill the unlock request");

        totalETH = totalETH.sub(_amount);
        _recipient.transfer(_amount);
        emit Unlock(_recipient, _amount);
    }
}
