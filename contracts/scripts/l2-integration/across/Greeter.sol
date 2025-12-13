// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

import {IERC20} from "openzeppelin/token/ERC20/IERC20.sol";
import {SafeERC20} from "openzeppelin/token/ERC20/utils/SafeERC20.sol";
import {Ownable} from "openzeppelin/access/Ownable.sol";
import {ISpokePool, IMessageHandler} from "./Interfaces.sol";
import {SwapParams, Instructions, Call} from "./Types.sol";

contract Greeter is Ownable {
    using SafeERC20 for IERC20;

    ISpokePool public immutable SPOKE_POOL;
    IMessageHandler public immutable HANDLER;
    address public remoteGreeter;
    mapping(address => string) public greetings;

    constructor(address _spokePool, address _handler) Ownable() {
        SPOKE_POOL = ISpokePool(_spokePool);
        HANDLER = IMessageHandler(_handler);
    }

    function setRemoteEndpoint(address _remote) public onlyOwner {
        remoteGreeter = _remote;
    }

    // Swap ERC20 token on Sepolia to get other token on L2, the fee should be calculated off-chain
    function swapTokenAndGreeting(
        SwapParams calldata params,
        string calldata _greeting,
        address recipient
    ) public {
        IERC20(params.inputToken).approve(address(SPOKE_POOL), params.inputAmount);

        Call[] memory calls = new Call[](1);
        bytes memory setGreetingCalldata = abi.encodeCall(this.setGreeting, (recipient, _greeting));
        calls[0] = Call({target: remoteGreeter, callData: setGreetingCalldata, value: 0});
        Instructions memory instructions =
            Instructions({calls: calls, fallbackRecipient: recipient});
        bytes memory message = abi.encode(instructions);

        SPOKE_POOL.deposit(
            bytes32(uint256(uint160(recipient))),
            bytes32(uint256(uint160(address(HANDLER)))),
            bytes32(uint256(uint160(params.inputToken))),
            bytes32(uint256(uint160(params.outputToken))),
            params.inputAmount,
            params.outputAmount,
            params.destinationChainId,
            bytes32(0), // exclusiveRelayer, zero means any relayer can fill
            uint32(block.timestamp - 600), // quoteTimestamp set to 10 minutes before now
            uint32(block.timestamp + 600), // fillDeadline set to 10 minutes after now
            0, // exclusivityDeadline, zero means no exclusivity
            message
        );
    }

    function setGreeting(address _sender, string memory _greeting) public {
        greetings[_sender] = _greeting;
    }
}
