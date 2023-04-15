// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.19;

import {AccessControl} from "openzeppelin/access/AccessControl.sol";
import {IOutboundQueueDelegate} from "./IOutboundQueueDelegate.sol";
import {IVault} from "./IVault.sol";
import {ParaID} from "./Types.sol";

contract OutboundQueueDelegate is IOutboundQueueDelegate, AccessControl {
    bytes32 public constant ADMIN_ROLE = keccak256("ADMIN_ROLE");

    mapping(ParaID dest => uint64) public nonce;

    IVault public vault;
    uint256 public fee;

    address immutable proxy;

    event FeeUpdated(uint256 fee);

    error InvalidSender();
    error FeePaymentToLow();
    error InvalidProxy();

    modifier onlyProxy {
        if (msg.sender != proxy) {
            revert InvalidProxy();
        }
        _;
    }

    constructor(address _proxy, IVault _vault, uint256 _fee) {
        _grantRole(ADMIN_ROLE, msg.sender);
        _setRoleAdmin(ADMIN_ROLE, ADMIN_ROLE);
        proxy = _proxy;
        vault = _vault;
        fee = _fee;
    }

    function submit(address, ParaID dest, bytes calldata) external payable onlyProxy returns (uint64) {
        if (msg.value < fee) {
            revert FeePaymentToLow();
        }

        nonce[dest] = nonce[dest] + 1;
        vault.deposit{value: msg.value}(dest);

        return nonce[dest];
    }

    function updateFee(uint256 newFee) external onlyRole(ADMIN_ROLE) {
        fee = newFee;
        emit FeeUpdated(fee);
    }
}
