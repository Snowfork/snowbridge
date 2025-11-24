// SPDX-License-Identifier: MIT
pragma solidity 0.8.28;

import "openzeppelin-contracts/contracts/token/ERC20/ERC20.sol";

contract FeeToken is ERC20 {
    constructor() ERC20("Fee Token", "FEE") {
        _mint(msg.sender, 1000000 * 10**18);
    }

    function transferFrom(address from, address to, uint256 amount) public override returns (bool) {
        uint256 fee = amount * 10 / 100; // 10% Fee
        uint256 amountAfterFee = amount - fee;

        _transfer(from, to, amountAfterFee);
        // Burn the fee to simulate loss
        _burn(from, fee);

        return true;
    }

    function deposit() public payable {
        _mint(msg.sender, msg.value);
    }
}
