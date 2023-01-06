// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.9;

import "./XcmAssetLookup.sol";

/// @dev Executes Xcm instructions.
contract XcmExecutor {
    /// @dev Represents the type of instruction.
    enum InstructionKind {
        /// @dev Transact allows abritrary call to another contract.
        Transact,
        /// @dev An asset has been reserved and can be minted.
        ReserveAssetsDeposited,
        /// @dev Deposit an asset to a destination account.
        DepositAsset
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

    /// @dev the data needed for reserve asset deposited instruction.
    struct ReserveAssetDepositedData {
        /// @dev The asset hash that was reserved.
        bytes32 assetHash;
        /// @dev The amount of that asset.
        uint256 amount;
    }

    /// @dev the data needed for deposit asset instruction.
    struct DepositAssetData {
        /// @dev The asset hash to deposit.
        bytes32 assetHash;
        /// @dev The amount of that asset to deposit.
        uint256 amount;
        /// @dev The destination account.
        address benificiary;
    }

    /// @dev The entry point for an payload.
    /// @param lookup lookup ERC20 tokens for an asset.
    /// @param instructions a list of instructions to execute.
    function execute(XcmAssetLookup lookup, Instruction[] memory instructions) external {
        // TODO: registers like origin, holding, etc...
        for (uint i = 0; i < instructions.length; i++) {
            if (instructions[i].kind == InstructionKind.Transact) {
                // 0x00 = Transact
                transact(abi.decode(instructions[i].arguments, (TransactData)));
            } else if (instructions[i].kind == InstructionKind.ReserveAssetsDeposited) {
                // 0x01 = ReserveAssetDeposited
                reserveAssetDeposited(
                    lookup,
                    abi.decode(instructions[i].arguments, (ReserveAssetDepositedData))
                );
            } else if (instructions[i].kind == InstructionKind.DepositAsset) {
                // 0x02 = DepositAsset
                depositAsset(lookup, abi.decode(instructions[i].arguments, (DepositAssetData)));
            } else {
                revert("Unknown instruction");
            }
        }
    }

    /// @dev single transact instruction.
    function transact(TransactData memory data) internal {
        (bool success, ) = data.target.call(data.payload);
        require(success, "transact failed");
    }

    /// @dev an asset that is held in reserved was deposited. This equates to an ERC20 mint.
    /// @param lookup lookup ERC20 tokens for an asset.
    /// @param data the instruction data.
    function reserveAssetDeposited(
        XcmAssetLookup lookup,
        ReserveAssetDepositedData memory data
    ) internal {
        require(data.amount > 0, "must reserve a positive amount");
        XcmFungibleAsset asset = lookup.lookup(data.assetHash);
        require(address(asset) != address(0), "cannot find asset");
        asset.mint(address(this), data.amount);
    }

    /// @dev an asset needs to be deposited out of the proxy. This equates to an ERC20 transfer from the proxy to some destination.
    /// @param lookup lookup ERC20 tokens for an asset.
    /// @param data the instruction data.
    function depositAsset(XcmAssetLookup lookup, DepositAssetData memory data) internal {
        require(data.amount > 0, "must reserve a positive amount");
        XcmFungibleAsset asset = lookup.lookup(data.assetHash);
        require(address(asset) != address(0), "cannot find asset");
        asset.transfer(data.benificiary, data.amount);
    }
}
