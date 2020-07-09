// SPDX-License-Identifier: MIT
pragma solidity ^0.6.2;

import "../node_modules/@openzeppelin/contracts/math/SafeMath.sol";
import "../node_modules/@openzeppelin/contracts/token/erc20/ERC20.sol";

contract Bank {
    using SafeMath for uint256;

    uint256 public nonce;
    uint256 public totalETH;
    mapping(address => uint256) public totalTokens;

    event Deposit(
        address _sender,    // Despositor's address on Ethereum
        bytes _recipient,   // Intended recipient's address on Polkadot
        address _tokenAddr, // Token address, empty for Ethereum deposits
        string _symbol,     // Asset's symbol
        uint256 _amount,    // Amount of Ethereum/tokens deposited
        uint256 _nonce      // Global nonce
    );

    constructor() public {
        nonce = 0;
    }

    function sendETH(
        bytes memory _recipient
    )
        public payable
    {
        require(msg.value > 0, "Value of transaction must be positive");

        // Increment locked Ethereum counter by this amount
        totalETH = totalETH.add(msg.value);
        // Increment global nonce
        nonce = nonce.add(1);

        emit Deposit(msg.sender, _recipient, address(0), "ETH", msg.value, nonce);
    }

    function sendERC20(
        bytes memory _recipient,
        address _tokenAddr,
        uint256 _amount
    )
        public
    {
       require(
            ERC20(_tokenAddr).transferFrom(msg.sender, address(this), _amount),
            "Contract token allowances insufficient to complete this lock request"
        );

        // Set symbol to the ERC20 token's symbol
        string memory symbol = ERC20(_tokenAddr).symbol();
        // Increment locked ERC20 token counter by this amount
        totalTokens[_tokenAddr] = totalTokens[_tokenAddr].add(_amount);
        // Increment global nonce
        nonce = nonce.add(1);

        emit Deposit(msg.sender, _recipient, _tokenAddr, symbol, _amount, nonce);
    }
}
