// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.23;

import "openzeppelin/token/ERC721/ERC721.sol";
import "openzeppelin/access/Ownable.sol";

contract MockNft is ERC721, Ownable {
    uint256 public totalMints = 0;

    constructor() ERC721("MyToken", "MTK") Ownable() {}

    function mint(address to) public {
        uint256 tokenId = totalMints;
        totalMints++;
        _safeMint(to, tokenId);
    }
}
