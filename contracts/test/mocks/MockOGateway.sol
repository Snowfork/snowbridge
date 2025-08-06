// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.25;

import {ParaID, OperatingMode} from "../../src/Types.sol";
import {CoreStorage} from "../../src/storage/CoreStorage.sol";
import {Verification} from "../../src/Verification.sol";
import {IInitializable} from "../../src/interfaces/IInitializable.sol";
import {UD60x18} from "prb/math/src/UD60x18.sol";

import {Gateway} from "../../src/Gateway.sol";

contract MockOGateway is Gateway {
    bool public commitmentsAreVerified;

    constructor(
        address beefyClient,
        address agentExecutor,
        ParaID bridgeHubParaID,
        bytes32 bridgeHubHubAgentID,
        uint8 foreignTokenDecimals,
        uint128 maxDestinationFee
    )
        Gateway(beefyClient, agentExecutor, bridgeHubParaID, bridgeHubHubAgentID, foreignTokenDecimals, maxDestinationFee)
    {}

    function agentExecutePublic(bytes calldata params) external {
        this.agentExecute(params);
    }

    function createAgentPublic(bytes calldata params) external {
        this.createAgent(params);
    }

    function upgradePublic(bytes calldata params) external {
        this.upgrade(params);
    }

    function createChannelPublic(bytes calldata params) external {
        this.createChannel(params);
    }

    function updateChannelPublic(bytes calldata params) external {
        this.updateChannel(params);
    }

    function setOperatingModePublic(bytes calldata params) external {
        this.setOperatingMode(params);
    }

    function transferNativeFromAgentPublic(bytes calldata params) external {
        this.transferNativeFromAgent(params);
    }

    function setCommitmentsAreVerified(bool value) external {
        commitmentsAreVerified = value;
    }

    function _verifyCommitment(bytes32 commitment, Verification.Proof calldata proof)
        internal
        view
        override
        returns (bool)
    {
        if (BEEFY_CLIENT != address(0)) {
            return super._verifyCommitment(commitment, proof);
        } else {
            // for unit tests, verification is set with commitmentsAreVerified
            return commitmentsAreVerified;
        }
    }

    function setTokenTransferFeesPublic(bytes calldata params) external {
        this.setTokenTransferFees(params);
    }

    function setPricingParametersPublic(bytes calldata params) external {
        this.setPricingParameters(params);
    }

    function registerForeignTokenPublic(bytes calldata params) external {
        this.registerForeignToken(params);
    }

    function mintForeignTokenPublic(bytes calldata params) external {
        this.mintForeignToken(params);
    }

    function transferNativeTokenPublic(bytes calldata params) external {
        this.transferNativeToken(params);
    }
}
