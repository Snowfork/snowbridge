// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

import "./XcmAssetLookup.sol";

/// @dev Executes Xcm instructions.
contract XcmExecutor {
    /// @dev Represents the type of instruction.
    enum InstructionKind {
        /// @dev Transact allows abritrary call to another contract.
        Transact
    }

    /// @dev A single instruction
    struct Instruction {
        /// @dev the type of instruction.
        InstructionKind kind;
        /// @dev the data provided for execution.
        bytes arguments;
    }

    /// @dev Data needed for xcm transact.
    struct TransactData {
        /// @dev The contract to call.
        address target;
        /// @dev The abi encoded payload with function selector.
        bytes payload;
    }

    /// @dev The entry point for an payload.
    function execute(XcmAssetLookup lookup, Instruction[] memory instructions) external {
        // TODO: registers like origin, holding, etc...
        for (uint i = 0; i < instructions.length; i++) {
            if (instructions[i].kind == InstructionKind.Transact) {
                // 0x00 = Transact
                transact(abi.decode(instructions[i].arguments, (TransactData)));
            } else {
                revert("Unknown instruction");
            }
        }
    }

    /// @dev single transact instruction.
    function transact(TransactData memory data) internal {
        (bool success, ) = data.target.call(data.payload);
        require(success, "Transact failed");
    }
}
