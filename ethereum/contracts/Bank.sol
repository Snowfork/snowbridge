// SPDX-License-Identifier: MIT
pragma solidity ^0.6.2;

import "../node_modules/@openzeppelin/contracts/math/SafeMath.sol";
import "./BankToken.sol";

contract Bank {
    using SafeMath for uint256;

    uint256 public nonce;
    uint256 public lockedEthereum;
    mapping(address => uint256) public lockedTokens;

    event Deposit(
        address _sender,  // Despositor's address on Ethereum
        bytes _recipient, // Intended recipient's address on Polkadot
        address _token,   // Token address, empty for Ethereum deposits
        string _symbol,   // Asset's symbol
        uint256 _amount,  // Amount of Ethereum/tokens deposited
        uint256 _nonce    // Global nonce
    );

    constructor() public {
        nonce = 0;
    }

    function depositETH(
        bytes memory _recipient
    )
        public payable
    {
        require(msg.value > 0, "Value of transaction must be positive");

        // Increment locked Ethereum counter by this amount
        lockedEthereum = lockedEthereum.add(msg.value);
        // Increment global nonce
        nonce = nonce.add(1);

        emit Deposit(msg.sender, _recipient, address(0), "ETH", msg.value, nonce);
    }

    function depositERC20(
        bytes memory _recipient,
        address _token,
        uint256 _amount
    )
        public
    {
       require(
            BankToken(_token).transferFrom(msg.sender, address(this), _amount),
            "Contract token allowances insufficient to complete this lock request"
        );

        // Set symbol to the ERC20 token's symbol
        string memory symbol = BankToken(_token).symbol();
        // Increment locked ERC20 token counter by this amount
        lockedTokens[_token] = lockedTokens[_token].add(_amount);
        // Increment global nonce
        nonce = nonce.add(1);

        emit Deposit(msg.sender, _recipient, _token, symbol, _amount, nonce);
    }
}
