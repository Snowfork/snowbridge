// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.33;

import {Gateway} from "../../src/Gateway.sol";
import {Functions} from "../../src/Gateway.sol";
import {Token} from "../../src/Token.sol";
import {ChannelID, ParaID, OperatingMode} from "../../src/Types.sol";

import {CoreStorage} from "../../src/storage/CoreStorage.sol";
import {Verification} from "../../src/Verification.sol";
import {IInitializable} from "../../src/interfaces/IInitializable.sol";

import {UD60x18} from "prb/math/src/UD60x18.sol";

import {Command as CommandV2, CommandKind} from "../../src/v2/Types.sol";
import {IGatewayV2} from "../../src/v2/IGateway.sol";
import {Agent} from "../../src/Agent.sol";
import {AgentExecutor} from "../../src/AgentExecutor.sol";
import {Constants} from "../../src/Constants.sol";
import {HandlersV2} from "../../src/v2/Handlers.sol";

contract MockGateway is Gateway {
    bool public commitmentsAreVerified;

    constructor(address beefyClient, address agentExecutor) Gateway(beefyClient, agentExecutor) {}

    function v1_handleAgentExecute_public(bytes calldata params) external {
        this.v1_handleAgentExecute(params);
    }

    function v1_handleUpgrade_public(bytes calldata params) external {
        this.v1_handleUpgrade(params);
    }

    function v1_handleSetOperatingMode_public(bytes calldata params) external {
        this.v1_handleSetOperatingMode(params);
    }

    function v1_handleSetTokenTransferFees_public(bytes calldata params) external {
        this.v1_handleSetTokenTransferFees(params);
    }

    function v1_handleSetPricingParameters_public(bytes calldata params) external {
        this.v1_handleSetPricingParameters(params);
    }

    function v1_handleUnlockNativeToken_public(bytes calldata params) external {
        this.v1_handleUnlockNativeToken(params);
    }

    function v1_handleRegisterForeignToken_public(bytes calldata params) external {
        this.v1_handleRegisterForeignToken(params);
    }

    function v1_handleMintForeignToken_public(ChannelID channelID, bytes calldata params)
        external
    {
        this.v1_handleMintForeignToken(channelID, params);
    }

    function setCommitmentsAreVerified(bool value) external {
        commitmentsAreVerified = value;
    }

    function prank_registerNativeToken(address token) external {
        Functions.registerNativeToken(token);
    }

    function prank_registerForeignToken(
        bytes32 foreignTokenID,
        string memory name,
        string memory symbol,
        uint8 decimals
    ) external returns (Token) {
        return Functions.registerForeignToken(foreignTokenID, name, symbol, decimals);
    }

    function _verifyCommitment(bytes32 commitment, Verification.Proof calldata proof, bool isV2)
        internal
        view
        override
        returns (bool)
    {
        if (BEEFY_CLIENT != address(0)) {
            return super._verifyCommitment(commitment, proof, isV2);
        } else {
            // for unit tests, verification is set with commitmentsAreVerified
            return commitmentsAreVerified;
        }
    }

    function transactionBaseGas() public pure returns (uint256) {
        return super.v1_transactionBaseGas();
    }

    // Dispatch a single V2 command (used by tests). Must be called via `this`.
    function v2_dispatchCommand(CommandV2 calldata command, bytes32 origin)
        external
        override
        onlySelf
    {
        if (command.kind == CommandKind.Upgrade) {
            HandlersV2.upgrade(command.payload);
        } else if (command.kind == CommandKind.SetOperatingMode) {
            HandlersV2.setOperatingMode(command.payload);
        } else if (command.kind == CommandKind.UnlockNativeToken) {
            HandlersV2.unlockNativeToken(AGENT_EXECUTOR, command.payload);
        } else if (command.kind == CommandKind.RegisterForeignToken) {
            HandlersV2.registerForeignToken(command.payload);
        } else if (command.kind == CommandKind.MintForeignToken) {
            HandlersV2.mintForeignToken(command.payload);
        } else if (command.kind == CommandKind.CallContract) {
            HandlersV2.callContract(origin, AGENT_EXECUTOR, command.payload);
        } else {
            revert IGatewayV2.InvalidCommand();
        }
    }

    function callDispatch(CommandV2 calldata command, bytes32 origin) external returns (bool) {
        // Mirror v2_dispatch per-command behavior: enforce gas budget and surface failure as false
        uint256 requiredGas = command.gas + DISPATCH_OVERHEAD_GAS_V2;
        if (gasleft() * 63 / 64 < requiredGas) {
            revert IGatewayV2.InsufficientGasLimit();
        }

        try this.v2_dispatchCommand{gas: requiredGas}(command, origin) {
            return true;
        } catch {
            return false;
        }
    }

    function deployAgent() external returns (address) {
        Agent a = new Agent(Constants.ASSET_HUB_AGENT_ID);
        return address(a);
    }

    function setAgentInStorage(address agent) external {
        CoreStorage.layout().agents[Constants.ASSET_HUB_AGENT_ID] = agent;
    }

    function callUnlockNativeToken(address executor, bytes calldata data) external {
        HandlersV2.unlockNativeToken(executor, data);
    }

    // Expose internal helper for testing
    function exposed_v1_transactionBaseGas() external pure returns (uint256) {
        return v1_transactionBaseGas();
    }

    // Helper to call vulnerable-onlySelf handler from within the contract (so msg.sender == this)
    function setOperatingMode(bytes calldata data) external {
        HandlersV2.setOperatingMode(data);
    }

    // Test helpers to manipulate storage for this gateway instance
    function setChannelAgent(ChannelID cid, address agent) external {
        CoreStorage.layout().channels[cid].agent = agent;
    }

    function setInboundNonce(uint64 n) external {
        CoreStorage.layout().inboundNonce.set(n);
    }
}
