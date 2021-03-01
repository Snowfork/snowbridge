// SPDX-License-Identifier: MIT
pragma solidity >=0.7.6;
pragma experimental ABIEncoderV2;

import "./BaseDOTApp.sol";

contract DotAppDecimals12 is BaseDOTApp {
    // Kusama (KSM) and Rococo (ROC) have 12 decimal places
    uint256 constant internal DECIMALS = 12;
    uint256 constant internal GRANULARITY = 10 ** (18 - DECIMALS);

    constructor(
        string memory _name,
        string memory _symbol,
        Channel memory _basic,
        Channel memory _incentivized
    )
        BaseDOTApp(_name, _symbol, _basic, _incentivized)
    { }

    /*
     * Smallest part of token that is not divisible when increasing precision
     * to 18 decimal places.
    */
    function granularity() pure internal override returns (uint256) {
        return GRANULARITY;
    }
}
