// SPDX-License-Identifier: MIT
pragma solidity >=0.7.6;
pragma experimental ABIEncoderV2;

import "./BaseDOTApp.sol";

contract DOTAppDecimals10 is BaseDOTApp {
    // Polkadot (DOT) has 10 decimal places
    uint256 constant internal DECIMALS = 10;
    uint256 constant internal GRANULARITY = 10 ** (18 - DECIMALS);

    constructor(
        string memory _name,
        string memory _symbol,
        Channel memory _basic,
        Channel memory _incentivized
    )
        BaseDOTApp(_name, _symbol, _basic, _incentivized)
    { }

    function granularity() pure internal override returns (uint256) {
        return GRANULARITY;
    }
}
