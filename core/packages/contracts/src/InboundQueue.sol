// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.19;

import "openzeppelin/utils/cryptography/MerkleProof.sol";
import "openzeppelin/access/AccessControl.sol";
import "./ParachainClient.sol";
import "./IRecipient.sol";
import "./IVault.sol";

contract InboundQueue is AccessControl {
    // Nonce for each origin
    mapping(bytes => uint64) public nonce;

    // Registered message handlers
    mapping(uint16 => Handler) handlers;

    // Light client message verifier
    IParachainClient public parachainClient;

    // Relayers are rewarded from this vault
    IVault public vault;

    // The relayer reward for submitting a message
    uint256 public reward;

    // The governance contract, which is a proxy for Polkadot governance, administers via this role
    bytes32 public constant ADMIN_ROLE = keccak256("ADMIN_ROLE");

    // Relayers must provide enough gas to cover message dispatch plus this buffer
    uint256 public constant GAS_BUFFER = 24000;

    // Inbound message from BridgeHub parachain
    struct Message {
        bytes origin;
        uint64 nonce;
        uint16 handler;
        bytes payload;
    }

    // Message handler entry
    struct Handler {
        // Address of message handler which implements IRecipient
        address recipient;
        // Amount of gas to forward to message handler
        uint32 gasToForward;
    }

    // The result of message dispatch
    struct DispatchResult {
        // Whether the dispatch succeeded
        bool succeeded;
        // Various error signifiers
        string errorReason;
        uint256 errorPanicCode;
        bytes errorReturnData;
    }

    event MessageDispatched(bytes origin, uint64 nonce, DispatchResult result);
    event HandlerUpdated(uint16 id, Handler handler);
    event ParachainClientUpdated(address parachainClient);
    event VaultUpdated(address vault);
    event RewardUpdated(uint256 reward);

    error InvalidProof();
    error InvalidNonce();
    error InvalidHandler();
    error NotEnoughGas();

    constructor(IParachainClient _parachainClient, IVault _vault, uint256 _reward) {
        _grantRole(ADMIN_ROLE, msg.sender);
        parachainClient = _parachainClient;
        vault = _vault;
        reward = _reward;
    }

    function submit(
        Message calldata message,
        bytes32[] calldata leafProof,
        bytes calldata headerProof
    ) external {
        bytes32 leafHash = keccak256(abi.encode(message));
        bytes32 commitment = MerkleProof.processProof(leafProof, leafHash);
        if (!parachainClient.verifyCommitment(commitment, headerProof)) {
            revert InvalidProof();
        }
        if (message.nonce != nonce[message.origin] + 1) {
            revert InvalidNonce();
        }

        // Increment nonce for origin.
        // This ensures messages are not replayed. It also ensures re-entrancy protection,
        // as a re-entrant call will be detected by the nonce check above.
        //
        // Sources of re-entrancy:
        // * The relayer which gets forwarded ETH as a reward for submission
        // * XCM::Transact calls to arbitrary untrusted contracts (not in scope for initial release)
        nonce[message.origin]++;

        // reward the relayer
        // Should revert if there are not enough funds. In which case, the origin
        // should top up the funds and have a relayer resend the message.
        vault.withdraw(message.origin, payable(msg.sender), reward);

        Handler memory handler = handlers[message.handler];
        if (address(handler.recipient) == address(0)) {
            revert InvalidHandler();
        }

        IRecipient recipient = IRecipient(handler.recipient);
        uint256 gasToForward = uint256(handler.gasToForward);

        // Ensure relayers pass enough gas for message to execute
        if (gasleft() < gasToForward + GAS_BUFFER) {
            revert NotEnoughGas();
        }

        DispatchResult memory result = DispatchResult(false, "", 0, hex"");

        // Forward message to handler for execution
        // Errors from the handler are ignored so as not to block the channel at the current nonce
        try recipient.handle{ gas: gasToForward }(message.origin, message.payload) {
            result.succeeded = true;
        } catch Error(string memory reason) {
            result.errorReason = reason;
        } catch Panic(uint256 errorCode) {
            result.errorPanicCode = errorCode;
        } catch (bytes memory returnData) {
            result.errorReturnData = returnData;
        }

        emit MessageDispatched(message.origin, message.nonce, result);
    }

    function updateHandler(uint16 id, Handler memory handler) external onlyRole(ADMIN_ROLE) {
        handlers[id] = handler;
        emit HandlerUpdated(id, handler);
    }

    function updateParachainClient(
        IParachainClient _parachainClient
    ) external onlyRole(ADMIN_ROLE) {
        parachainClient = _parachainClient;
        emit ParachainClientUpdated(address(_parachainClient));
    }

    function updateVault(IVault _vault) external onlyRole(ADMIN_ROLE) {
        vault = _vault;
        emit VaultUpdated(address(_vault));
    }

    function updateReward(uint256 _reward) external onlyRole(ADMIN_ROLE) {
        reward = _reward;
        emit RewardUpdated(_reward);
    }
}
