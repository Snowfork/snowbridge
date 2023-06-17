// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.20;

import {AccessControl} from "openzeppelin/access/AccessControl.sol";

contract Registry is AccessControl {
    bytes32 public constant ADMIN_ROLE = keccak256("ADMIN_ROLE");
    bytes32 public constant REGISTER_ROLE = keccak256("REGISTER_ROLE");

    mapping(bytes32 contractID => address) public registry;

    constructor() {
        _grantRole(ADMIN_ROLE, msg.sender);
        _setRoleAdmin(ADMIN_ROLE, ADMIN_ROLE);
        _setRoleAdmin(REGISTER_ROLE, ADMIN_ROLE);
    }

    function registerContract(bytes32 contractID, address contractAddress) external onlyRole(REGISTER_ROLE) {
        registry[contractID] = contractAddress;
    }

    function lookupContract(bytes32 contractID) external view returns (address) {
        return registry[contractID];
    }
}
