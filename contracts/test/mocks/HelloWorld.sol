// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.33;

interface IV1HandlerGateway {
    function v1_handleUnlockNativeToken(bytes calldata params) external;
}

struct V2Command {
    uint8 kind;
    uint64 gas;
    bytes payload;
}

interface IV2DispatchGateway {
    function v2_dispatchCommand(V2Command calldata command, bytes32 origin) external;
}

contract HelloWorld {
    event SaidHello(string indexed message, address sender, address self);

    error Unauthorized();

    function sayHello(string memory _text) public payable {
        string memory fullMessage = string(abi.encodePacked("Hello there, ", _text));
        emit SaidHello(fullMessage, msg.sender, address(this));
    }

    function callV1HandleUnlockNativeToken(bytes memory params) public {
        IV1HandlerGateway(msg.sender).v1_handleUnlockNativeToken(params);
    }

    function delegateCallV1HandleUnlockNativeToken(address target, bytes memory params) public {
        (bool ok,) = target.delegatecall(
            abi.encodeWithSelector(IV1HandlerGateway.v1_handleUnlockNativeToken.selector, params)
        );
        require(ok);
    }

    function callV2DispatchUnlockNativeToken(bytes memory params, bytes32 origin) public {
        V2Command memory command = V2Command({kind: 2, gas: 500_000, payload: params});
        IV2DispatchGateway(msg.sender).v2_dispatchCommand(command, origin);
    }

    function delegateCallV2DispatchUnlockNativeToken(
        address target,
        bytes memory params,
        bytes32 origin
    ) public {
        V2Command memory command = V2Command({kind: 2, gas: 500_000, payload: params});
        (bool ok,) = target.delegatecall(
            abi.encodeWithSelector(IV2DispatchGateway.v2_dispatchCommand.selector, command, origin)
        );
        require(ok);
    }

    function revertUnauthorized() public pure {
        revert Unauthorized();
    }

    function retBomb() public pure returns (bytes memory) {
        assembly {
            return(1, 3000000)
        }
    }
}
