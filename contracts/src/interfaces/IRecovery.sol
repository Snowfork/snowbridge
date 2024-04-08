// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.23;

interface IRecovery {
    /// The Gateway has recovered with a new implementation contract
    event Recovered(address impl, bytes32 initializerParamsHash);

    /// The ability to rescue the gateway has been relinquished
    event Renounced();

    /// Recover the Gateway with a new implementation contract
    function recover(address impl, bytes32 implCodeHash, bytes calldata initializerParams) external;

    /// Relinquish rescue ability
    function renounce() external;

    /// Query the trusted recovery operator. If the return value is `address(0)`,
    /// the rescue ability has been permanently disabled.
    function recoveryOperator() external view returns (address);
}
