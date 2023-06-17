// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.20;

import {AccessControl} from "openzeppelin/access/AccessControl.sol";
import {IOutboundQueue} from "./IOutboundQueue.sol";
import {Registry} from "./Registry.sol";
import {Vault} from "./Vault.sol";
import {ParaID} from "./Types.sol";
import {Auth} from "./Auth.sol";
import {RegistryLookup} from "./RegistryLookup.sol";

contract OutboundQueue is Auth, RegistryLookup, IOutboundQueue {
    bytes32 public constant SUBMIT_ROLE = keccak256("SUBMIT_ROLE");

    mapping(ParaID dest => uint64) public nonce;

    Vault public immutable vault;
    uint256 public fee;

    event FeeUpdated(uint256 fee);

    error FeePaymentToLow();

    constructor(Registry registry, Vault _vault, uint256 _fee) RegistryLookup(registry) {
        _setRoleAdmin(SUBMIT_ROLE, ADMIN_ROLE);
        vault = _vault;
        fee = _fee;
    }

    function submit(ParaID dest, bytes calldata payload) external payable onlyRole(SUBMIT_ROLE) {
        if (msg.value < fee) {
            revert FeePaymentToLow();
        }
        nonce[dest] = nonce[dest] + 1;
        vault.deposit{value: msg.value}(dest);
        emit Message(dest, nonce[dest], payload);
    }

    function updateFee(uint256 newFee) external onlyRole(ADMIN_ROLE) {
        fee = newFee;
        emit FeeUpdated(fee);
    }
}
