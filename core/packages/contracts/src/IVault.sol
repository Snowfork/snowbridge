// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.19;

import {ParaID} from "./Types.sol";

interface IVault {
    function deposit(ParaID sovereign) external payable;

    function withdraw(ParaID sovereign, address payable recipient, uint256 amount) external;
}
