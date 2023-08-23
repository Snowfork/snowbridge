// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.20;

import {Gateway} from "../../src/Gateway.sol";
import {ParaID, OperatingMode} from "../../src/Types.sol";
import {CoreStorage} from "../../src/storage/CoreStorage.sol";

contract GatewayMock is Gateway {
    constructor(
        address beefyClient,
        address agentExecutor,
        uint256 dispatchGas,
        ParaID bridgeHubParaID,
        bytes32 bridgeHubHubAgentID,
        ParaID assetHubParaID,
        bytes32 assetHubHubAgentID,
        ParaID templateParaID,
        bytes32 templateAgentID,
        bytes2 createTokenCallID,
        bytes32 create2Salt
    )
        Gateway(
            beefyClient,
            agentExecutor,
            dispatchGas,
            bridgeHubParaID,
            bridgeHubHubAgentID,
            assetHubParaID,
            assetHubHubAgentID,
            templateParaID,
            templateAgentID,
            createTokenCallID,
            create2Salt
        )
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
}

library AdditionalStorage {
    struct Layout {
        uint256 value;
    }

    bytes32 internal constant SLOT = keccak256("org.snowbridge.storage.additionalStorage");

    function layout() internal pure returns (Layout storage sp) {
        bytes32 slot = SLOT;
        assembly {
            sp.slot := slot
        }
    }
}

// Used to test upgrades.
contract GatewayV2 {
    // Reinitialize gateway with some additional storage fields
    function initialize(bytes memory params) external {
        AdditionalStorage.Layout storage $ = AdditionalStorage.layout();

        uint256 value = abi.decode(params, (uint256));

        if (value == 666) {
            revert("initialize failed");
        }

        $.value = value;
    }

    function getValue() external view returns (uint256) {
        return AdditionalStorage.layout().value;
    }
}
