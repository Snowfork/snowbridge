// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

interface IVault {
    function deposit(bytes calldata sovereign) external payable;
    function withdraw(bytes calldata sovereign, address payable recipient, uint256 amount) external;
}
