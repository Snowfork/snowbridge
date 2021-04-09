// SPDX-License-Identifier: MIT
pragma solidity ^0.7.6;

import "../../node_modules/@openzeppelin/contracts/token/ERC721/ERC721.sol";

contract TestToken721 is ERC721 {
    constructor(string memory _name, string memory _symbol) ERC721(_name, _symbol) {}

    function mint(address to, uint256 tokenId) public {
        _mint(to, tokenId);
    }

    function mintWithTokenURI(address to, uint256 tokenId, string memory tokenURI) public {
        mint(to, tokenId);
        _setTokenURI(tokenId, tokenURI);
    }
}
