// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.20;

interface IGateway {
    function submitOutbound(bytes32 laneID, bytes calldata message)
}
