// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "@openzeppelin/contracts/token/ERC20/extensions/ERC20Burnable.sol";
import "@openzeppelin/contracts/access/Ownable.sol";

import "./XcmProxy.sol";

/// @title Represents a fungible asset from substrate.
contract XcmFungibleAsset is ERC20, Ownable {
    /// @dev initializes asset with an owner.
    constructor() ERC20("", "") {
    }

    function decimals() public view virtual override returns (uint8) {
        return 10;
    }

    /// @dev mints the asset.
    /// @param _account the account to mint to.
    /// @param _amount the amount to mint.
    function mint(address _account, uint256 _amount) public onlyOwner {
        _mint(_account, _amount);
    }

    /// @dev mints the asset.
    /// @param _account the account to mint to.
    /// @param _amount the amount to mint.
    function burn(address _account, uint256 _amount) public onlyOwner {
        _burn(_account, _amount);
    }
}
