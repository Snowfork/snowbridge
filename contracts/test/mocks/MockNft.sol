// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.23;

import "openzeppelin/token/ERC721/ERC721.sol";
import "openzeppelin/access/Ownable.sol";

contract MockNft is ERC721, Ownable {
    constructor() ERC721("MyToken", "MTK") Ownable() {}
}
