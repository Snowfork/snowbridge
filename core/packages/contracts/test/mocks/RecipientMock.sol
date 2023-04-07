// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.19;

import {IRecipient} from "../../src/IRecipient.sol";
import {ParaID} from "../../src/Types.sol";

contract RecipientMock is IRecipient {
    bool shouldFail;
    bool shouldPanic;
    bool shouldConsumeAllGas;

    error Failed();

    event Called();

    function setShouldFail() external {
        shouldFail = true;
        shouldPanic = false;
        shouldConsumeAllGas = false;
    }

    function setShouldPanic() external {
        shouldFail = false;
        shouldPanic = true;
        shouldConsumeAllGas = false;
    }

    function setShouldConsumeAllGas() external {
        shouldFail = false;
        shouldPanic = false;
        shouldConsumeAllGas = true;
    }

    function handle(ParaID, bytes calldata) external {
        if (shouldFail) {
            revert("failed");
        }
        if (shouldPanic) {
            assert(false);
        }
        if (shouldConsumeAllGas) {
            while (true) {}
        }
        emit Called();
    }
}
