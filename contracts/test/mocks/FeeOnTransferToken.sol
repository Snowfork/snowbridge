// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

import {ERC20} from "openzeppelin/token/ERC20/ERC20.sol";

/// @dev Test token that burns a fee on every transfer to simulate fee-on-transfer tokens.
contract FeeOnTransferToken is ERC20 {
    uint256 public immutable feeBps;

    constructor(string memory name, string memory symbol, uint256 feeBps_) ERC20(name, symbol) {
        feeBps = feeBps_;
        _mint(msg.sender, 1_000_000 ether);
    }

    function mint(address to, uint256 amount) external {
        _mint(to, amount);
    }

    function _transfer(address from, address to, uint256 amount) internal override {
        uint256 fee = (amount * feeBps) / 10_000;
        uint256 amountAfterFee = amount - fee;

        super._transfer(from, to, amountAfterFee);
        if (fee > 0) {
            _burn(from, fee);
        }
    }
}
