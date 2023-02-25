// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

import "@openzeppelin/contracts/access/AccessControl.sol";
import "./IOutboundChannel.sol";

// BasicOutboundChannel is a basic channel that just sends messages with a nonce.
contract OutboundChannel is IOutboundChannel, AccessControl {

    bytes32 public constant SUBMIT_ROLE = keccak256("SUBMIT_ROLE");

    mapping(bytes32 => uint64) public nonce;

    event Message(bytes32 dest, uint64 nonce, bytes payload);

    constructor() {
        _setupRole(DEFAULT_ADMIN_ROLE, msg.sender);
    }

    function authorizeApp(address app) external onlyRole(DEFAULT_ADMIN_ROLE) {
        grantRole(SUBMIT_ROLE, app);
    }

    function submit(bytes32 dest, bytes calldata payload) external override onlyRole(SUBMIT_ROLE) {
        nonce[dest] = nonce[dest] + 1;
        emit Message(dest, nonce[dest], payload);
    }
}
