// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.20;

import {Gateway} from "../../src/Gateway.sol";
import {ParaID, OperatingMode} from "../../src/Types.sol";
import {Initializable} from "../../src/Initializable.sol";
import {CoreStorage} from "../../src/storage/CoreStorage.sol";

contract GatewayMock is Gateway {
    function handleAgentExecutePublic(bytes calldata params) external {
        this.handleAgentExecute(params);
    }

    function handleCreateAgentPublic(bytes calldata params) external {
        this.handleCreateAgent(params);
    }

    function handleUpgradePublic(bytes calldata params) external {
        this.handleUpgrade(params);
    }

    function handleCreateChannelPublic(bytes calldata params) external {
        this.handleCreateChannel(params);
    }

    function handleUpdateChannelPublic(bytes calldata params) external {
        this.handleUpdateChannel(params);
    }

    function handleSetOperatingModePublic(bytes calldata params) external {
        this.handleSetOperatingMode(params);
    }

    function setAgentExecutor(address agentExecutor) external {
        CoreStorage.layout().agentExecutor = agentExecutor;
    }

    function setOperatingMode(OperatingMode mode) external {
        CoreStorage.layout().mode = mode;
    }

    function setChannelOperatingMode(ParaID paraID, OperatingMode mode) external {
        CoreStorage.layout().channels[paraID].mode = mode;
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
contract GatewayV2 is Gateway {
    // Reinitialize gateway with some additional storage fields
    function initializeV2() external reinitializer(2) {
        AdditionalStorage.Layout storage $ = AdditionalStorage.layout();
        $.value = 42;
    }

    function getValue() external view returns (uint256) {
        return AdditionalStorage.layout().value;
    }
}
