// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

import {Gateway} from "../../src/Gateway.sol";
import {ChannelID, ParaID, OperatingMode} from "../../src/Types.sol";
import {CoreStorage} from "../../src/storage/CoreStorage.sol";
import {ParachainVerification} from "../../src/ParachainVerification.sol";
import {BeefyVerification} from "../../src/BeefyVerification.sol";
import {IInitializable} from "../../src/interfaces/IInitializable.sol";

import {UD60x18} from "prb/math/src/UD60x18.sol";

contract MockGateway is Gateway {
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

    function setCommitmentsAreVerified(bool value) external {
        commitmentsAreVerified = value;
    }

    function _buildHeadersRoot(bytes32, ParachainVerification.Proof calldata)
        internal
        pure
        override
        returns (bytes32)
    {
        return bytes32(0);
    }

    function _verifyBeefyProof(bytes32 parachainHeadersRoot, BeefyVerification.Proof calldata beefyProof)
        internal
        view
        override
        returns (bool)
    {
        if (BEEFY_CLIENT != address(0)) {
            return super._verifyBeefyProof(parachainHeadersRoot, beefyProof);
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

    function mintForeignTokenPublic(ChannelID channelID, bytes calldata params) external {
        this.mintForeignToken(channelID, params);
    }

    function transferNativeTokenPublic(bytes calldata params) external {
        this.transferNativeToken(params);
    }
}
