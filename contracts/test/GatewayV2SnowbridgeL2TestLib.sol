// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.33;

import {SnowbridgeL1Adaptor} from "../src/l2-integration/SnowbridgeL1Adaptor.sol";
import {DepositParams} from "../src/l2-integration/Types.sol";
import {CallContractParams, UnlockNativeTokenParams} from "../src/v2/Types.sol";
import {CommandV2, CommandKind, InboundMessageV2} from "../src/Types.sol";
import {Constants} from "../src/Constants.sol";

library SnowbridgeL2TestLib {
    function makeUnlockTokenCommand(address token, address recipient, uint128 amount)
        internal
        pure
        returns (CommandV2 memory)
    {
        UnlockNativeTokenParams memory params =
            UnlockNativeTokenParams({token: token, recipient: recipient, amount: amount});
        return CommandV2({
            kind: CommandKind.UnlockNativeToken,
            gas: 500_000,
            payload: abi.encode(params)
        });
    }

    function makeDepositTokenCommand(
        address adaptor,
        DepositParams memory params,
        address recipient,
        bytes32 topic
    ) internal pure returns (CommandV2[] memory) {
        bytes memory data = abi.encodeWithSelector(
            SnowbridgeL1Adaptor.depositToken.selector, params, recipient, topic
        );
        CallContractParams memory callParams =
            CallContractParams({target: adaptor, data: data, value: 0});
        CommandV2[] memory commands = new CommandV2[](1);
        commands[0] = CommandV2({
            kind: CommandKind.CallContract, gas: 500_000, payload: abi.encode(callParams)
        });
        return commands;
    }

    /// Builds commands: first UnlockNativeToken to prefund the adaptor, then CallContract to depositToken.
    function makeDepositTokenCommandWithPrefund(
        address adaptor,
        DepositParams memory params,
        address recipient,
        bytes32 topic,
        address prefundToken,
        uint128 prefundAmount
    ) internal pure returns (CommandV2[] memory) {
        CommandV2[] memory commands = new CommandV2[](2);
        commands[0] = makeUnlockTokenCommand(prefundToken, adaptor, prefundAmount);
        bytes memory data = abi.encodeWithSelector(
            SnowbridgeL1Adaptor.depositToken.selector, params, recipient, topic
        );
        CallContractParams memory callParams =
            CallContractParams({target: adaptor, data: data, value: 0});
        commands[1] = CommandV2({
            kind: CommandKind.CallContract, gas: 500_000, payload: abi.encode(callParams)
        });
        return commands;
    }

    function makeDepositParamsToken(address inputToken, uint256 inputAmount, uint256 outputAmount)
        internal
        pure
        returns (DepositParams memory)
    {
        return DepositParams({
            inputToken: inputToken,
            outputToken: address(0x1234),
            inputAmount: inputAmount,
            outputAmount: outputAmount,
            destinationChainId: 8453,
            fillDeadlineBuffer: 600
        });
    }

    function makeDepositParamsNativeEther(uint256 inputAmount, uint256 outputAmount)
        internal
        pure
        returns (DepositParams memory)
    {
        return DepositParams({
            inputToken: address(0),
            outputToken: address(0x1234),
            inputAmount: inputAmount,
            outputAmount: outputAmount,
            destinationChainId: 8453,
            fillDeadlineBuffer: 600
        });
    }

    function makeDepositNativeEtherCommand(
        address adaptor,
        DepositParams memory params,
        address recipient,
        bytes32 topic
    ) internal pure returns (CommandV2 memory) {
        bytes memory data = abi.encodeWithSelector(
            SnowbridgeL1Adaptor.depositNativeEther.selector, params, recipient, topic
        );
        CallContractParams memory callParams =
            CallContractParams({target: adaptor, data: data, value: 0});
        return CommandV2({
            kind: CommandKind.CallContract, gas: 500_000, payload: abi.encode(callParams)
        });
    }

    /// Builds commands: UnlockNativeToken (native ETH) to prefund the adaptor, then CallContract depositNativeEther.
    function makeDepositNativeEtherCommandWithPrefund(
        address adaptor,
        DepositParams memory params,
        address recipient,
        bytes32 topic,
        uint128 prefundAmount
    ) internal pure returns (CommandV2[] memory) {
        CommandV2[] memory commands = new CommandV2[](2);
        commands[0] = makeUnlockTokenCommand(address(0), adaptor, prefundAmount);
        commands[1] = makeDepositNativeEtherCommand(adaptor, params, recipient, topic);
        return commands;
    }

    function makeDepositNativeEtherMessageWithPrefund(
        address adaptor,
        DepositParams memory params,
        address recipient,
        bytes32 topic,
        uint128 prefundAmount
    ) internal pure returns (InboundMessageV2 memory) {
        return InboundMessageV2({
            origin: Constants.ASSET_HUB_AGENT_ID,
            nonce: 1,
            topic: topic,
            commands: makeDepositNativeEtherCommandWithPrefund(
                adaptor, params, recipient, topic, prefundAmount
            )
        });
    }

    function makeDepositTokenMessage(
        address adaptor,
        DepositParams memory params,
        address recipient,
        bytes32 topic
    ) internal pure returns (InboundMessageV2 memory) {
        return InboundMessageV2({
            origin: Constants.ASSET_HUB_AGENT_ID,
            nonce: 1,
            topic: topic,
            commands: makeDepositTokenCommand(adaptor, params, recipient, topic)
        });
    }

    function makeDepositTokenMessageWithPrefund(
        address adaptor,
        DepositParams memory params,
        address recipient,
        bytes32 topic,
        address prefundToken,
        uint128 prefundAmount
    ) internal pure returns (InboundMessageV2 memory) {
        return InboundMessageV2({
            origin: Constants.ASSET_HUB_AGENT_ID,
            nonce: 1,
            topic: topic,
            commands: makeDepositTokenCommandWithPrefund(
                adaptor, params, recipient, topic, prefundToken, prefundAmount
            )
        });
    }
}
