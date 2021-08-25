// SPDX-License-Identifier: Apache-2.0
pragma solidity >=0.8.5;
pragma experimental ABIEncoderV2;

import "@openzeppelin/contracts/token/ERC777/ERC777.sol";
import "@openzeppelin/contracts/access/Ownable.sol";

contract WrappedToken is ERC777, Ownable {

    constructor(
        string memory _name,
        string memory _symbol,
        address[] memory _defaultOperators
    )
        ERC777(_name, _symbol, _defaultOperators)
    { }

    function burn(address sender, uint256 amount, bytes memory data) external onlyOwner {
        _burn(sender, amount, data, "");
    }

    function mint(address recipient, uint256 amount, bytes memory data) external onlyOwner {
        _mint(recipient, amount, data, "");
    }

    // Don't allow users to directly burn their wrapped tokens via the IERC777 burn API, as it won't redeem
    // the native tokens on substrate.

    function burn(uint256, bytes memory) public pure override  {
        revert("not-supported");
    }

    function operatorBurn(address, uint256, bytes memory, bytes memory) public pure override {
        revert("not-supported");
    }
}
