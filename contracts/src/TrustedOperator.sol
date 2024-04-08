// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.23;

import {TrustedOperatorStorage} from "./storage/TrustedOperatorStorage.sol";

library TrustedOperator {
    function setOperator(address _operator) internal {
        TrustedOperatorStorage.Layout storage $ = TrustedOperatorStorage.layout();
        $.operator = _operator;
    }

    function renounce() internal {
        TrustedOperatorStorage.Layout storage $ = TrustedOperatorStorage.layout();
        delete $.operator;
    }

    function operator() internal view returns (address) {
        TrustedOperatorStorage.Layout storage $ = TrustedOperatorStorage.layout();
        return $.operator;
    }
}
