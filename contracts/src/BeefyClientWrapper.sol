// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2023 Snowfork <hello@snowfork.com>
pragma solidity 0.8.28;

import {IBeefyClient} from "./interfaces/IBeefyClient.sol";
import {IInitializable} from "./interfaces/IInitializable.sol";
import {IUpgradable} from "./interfaces/IUpgradable.sol";
import {ERC1967} from "./utils/ERC1967.sol";

/**
 * @title BeefyClientWrapper
 * @dev Forwards BeefyClient submissions and refunds gas costs to whitelisted relayers.
 * Implements soft round-robin scheduling to prevent competition.
 */
contract BeefyClientWrapper is IInitializable, IUpgradable {
    event RelayerAdded(address indexed relayer);
    event RelayerRemoved(address indexed relayer);
    event GasCredited(address indexed relayer, bytes32 indexed commitmentHash, uint256 gasUsed);
    event SubmissionRefunded(address indexed relayer, uint256 amount, uint256 totalGasUsed);
    event TurnAdvanced(uint256 indexed newTurnIndex, address indexed nextRelayer);
    event FundsDeposited(address indexed depositor, uint256 amount);
    event FundsWithdrawn(address indexed recipient, uint256 amount);

    error Unauthorized();
    error NotARelayer();
    error NotYourTurn();
    error InsufficientBlockIncrement();
    error RelayerAlreadyExists();
    error RelayerNotFound();
    error NoRelayers();
    error InvalidAddress();
    error InvalidTicket();
    error NotTicketOwner();
    error TransferFailed();
    error AlreadyInitialized();
    error TicketAlreadyActive();

    address public owner;
    IBeefyClient public beefyClient;
    uint256 public maxGasPrice;
    address[] public relayers;
    mapping(address => bool) public isRelayer;
    mapping(address => uint256) private relayerIndex;
    uint256 public currentTurnIndex;
    uint256 public lastSubmissionBlock;
    uint256 public gracePeriodBlocks;
    uint256 public minBlockIncrement;
    mapping(bytes32 => address) public ticketOwner;
    mapping(address => bytes32) public activeTicket;
    mapping(bytes32 => uint256) public creditedGas;
    uint256 public maxRefundAmount;
    bool private initialized;

    function initialize(bytes calldata data) external override {
        if (initialized) {
            revert AlreadyInitialized();
        }
        initialized = true;

        (
            address _beefyClient,
            address _owner,
            uint256 _maxGasPrice,
            uint256 _gracePeriodBlocks,
            uint256 _minBlockIncrement,
            uint256 _maxRefundAmount
        ) = abi.decode(data, (address, address, uint256, uint256, uint256, uint256));

        if (_beefyClient == address(0) || _owner == address(0)) {
            revert InvalidAddress();
        }

        beefyClient = IBeefyClient(_beefyClient);
        owner = _owner;
        maxGasPrice = _maxGasPrice;
        gracePeriodBlocks = _gracePeriodBlocks;
        minBlockIncrement = _minBlockIncrement;
        maxRefundAmount = _maxRefundAmount;
    }

    /* Beefy Client Proxy Functions */

    function submitInitial(
        IBeefyClient.Commitment calldata commitment,
        uint256[] calldata bitfield,
        IBeefyClient.ValidatorProof calldata proof
    ) external {
        _checkEligibleRelayer();
        if (activeTicket[msg.sender] != bytes32(0)) {
            revert TicketAlreadyActive();
        }
        uint256 startGas = gasleft();

        uint64 latestBlock = beefyClient.latestBeefyBlock();
        if (commitment.blockNumber < latestBlock + minBlockIncrement) {
            revert InsufficientBlockIncrement();
        }

        beefyClient.submitInitial(commitment, bitfield, proof);

        bytes32 commitmentHash = keccak256(abi.encode(commitment));
        ticketOwner[commitmentHash] = msg.sender;
        activeTicket[msg.sender] = commitmentHash;

        _creditGas(startGas, commitmentHash);
    }

    function commitPrevRandao(bytes32 commitmentHash) external {
        uint256 startGas = gasleft();

        if (ticketOwner[commitmentHash] != msg.sender) {
            revert NotTicketOwner();
        }

        beefyClient.commitPrevRandao(commitmentHash);

        _creditGas(startGas, commitmentHash);
    }

    function submitFinal(
        IBeefyClient.Commitment calldata commitment,
        uint256[] calldata bitfield,
        IBeefyClient.ValidatorProof[] calldata proofs,
        IBeefyClient.MMRLeaf calldata leaf,
        bytes32[] calldata leafProof,
        uint256 leafProofOrder
    ) external {
        uint256 startGas = gasleft();

        bytes32 commitmentHash = keccak256(abi.encode(commitment));
        if (ticketOwner[commitmentHash] != msg.sender) {
            revert NotTicketOwner();
        }

        beefyClient.submitFinal(commitment, bitfield, proofs, leaf, leafProof, leafProofOrder);

        uint256 previousGas = creditedGas[commitmentHash];
        delete creditedGas[commitmentHash];
        delete ticketOwner[commitmentHash];
        delete activeTicket[msg.sender];

        _advanceTurn();
        _refundGas(startGas, previousGas);
    }

    function createFinalBitfield(bytes32 commitmentHash, uint256[] calldata bitfield)
        external
        view
        returns (uint256[] memory)
    {
        return beefyClient.createFinalBitfield(commitmentHash, bitfield);
    }

    function latestBeefyBlock() external view returns (uint64) {
        return beefyClient.latestBeefyBlock();
    }

    function createInitialBitfield(uint256[] calldata bitsToSet, uint256 length)
        external
        view
        returns (uint256[] memory)
    {
        return beefyClient.createInitialBitfield(bitsToSet, length);
    }

    function randaoCommitDelay() external view returns (uint256) {
        return beefyClient.randaoCommitDelay();
    }

    function currentValidatorSet()
        external
        view
        returns (uint128 id, uint128 length, bytes32 root)
    {
        return beefyClient.currentValidatorSet();
    }

    function nextValidatorSet()
        external
        view
        returns (uint128 id, uint128 length, bytes32 root)
    {
        return beefyClient.nextValidatorSet();
    }

    function clearTicket() external {
        bytes32 commitmentHash = activeTicket[msg.sender];
        if (commitmentHash == bytes32(0)) {
            revert InvalidTicket();
        }

        // Credited gas is forfeited when clearing a ticket
        delete creditedGas[commitmentHash];
        delete ticketOwner[commitmentHash];
        delete activeTicket[msg.sender];
    }

    /* Internal Functions */

    function _checkOwner() internal view {
        if (msg.sender != owner) {
            revert Unauthorized();
        }
    }

    function _checkEligibleRelayer() internal view {
        if (!isRelayer[msg.sender]) {
            revert NotARelayer();
        }

        if (relayers.length > 0) {
            address assignedRelayer = relayers[currentTurnIndex % relayers.length];
            bool isAssignedRelayer = (msg.sender == assignedRelayer);
            bool gracePeriodActive = block.number > lastSubmissionBlock + gracePeriodBlocks;

            if (!isAssignedRelayer && !gracePeriodActive) {
                revert NotYourTurn();
            }
        }
    }

    function _creditGas(uint256 startGas, bytes32 commitmentHash) internal {
        uint256 gasUsed = startGas - gasleft() + 21000;
        creditedGas[commitmentHash] += gasUsed;
        emit GasCredited(msg.sender, commitmentHash, gasUsed);
    }

    function _refundGas(uint256 startGas, uint256 previousGas) internal {
        uint256 currentGas = startGas - gasleft() + 21000;
        uint256 totalGasUsed = currentGas + previousGas;
        uint256 effectiveGasPrice = tx.gasprice < maxGasPrice ? tx.gasprice : maxGasPrice;
        uint256 refundAmount = totalGasUsed * effectiveGasPrice;

        // Cap the refund to prevent draining the contract
        if (refundAmount > maxRefundAmount) {
            refundAmount = maxRefundAmount;
        }

        if (address(this).balance >= refundAmount) {
            (bool success,) = payable(msg.sender).call{value: refundAmount}("");
            if (success) {
                emit SubmissionRefunded(msg.sender, refundAmount, totalGasUsed);
            }
        }
    }

    function _advanceTurn() internal {
        currentTurnIndex++;
        lastSubmissionBlock = block.number;

        if (relayers.length > 0) {
            address nextRelayer = relayers[currentTurnIndex % relayers.length];
            emit TurnAdvanced(currentTurnIndex, nextRelayer);
        }
    }

    /* Admin Functions */

    function addRelayer(address relayer) external {
        _checkOwner();
        if (relayer == address(0)) {
            revert InvalidAddress();
        }
        if (isRelayer[relayer]) {
            revert RelayerAlreadyExists();
        }

        relayerIndex[relayer] = relayers.length;
        relayers.push(relayer);
        isRelayer[relayer] = true;

        emit RelayerAdded(relayer);
    }

    function removeRelayer(address relayer) external {
        _checkOwner();
        if (!isRelayer[relayer]) {
            revert RelayerNotFound();
        }

        uint256 indexToRemove = relayerIndex[relayer];
        uint256 lastIndex = relayers.length - 1;

        if (indexToRemove != lastIndex) {
            address lastRelayer = relayers[lastIndex];
            relayers[indexToRemove] = lastRelayer;
            relayerIndex[lastRelayer] = indexToRemove;
        }

        relayers.pop();
        delete isRelayer[relayer];
        delete relayerIndex[relayer];

        emit RelayerRemoved(relayer);
    }

    function setMaxGasPrice(uint256 _maxGasPrice) external {
        _checkOwner();
        maxGasPrice = _maxGasPrice;
    }

    function setGracePeriod(uint256 _gracePeriodBlocks) external {
        _checkOwner();
        gracePeriodBlocks = _gracePeriodBlocks;
    }

    function setMinBlockIncrement(uint256 _minBlockIncrement) external {
        _checkOwner();
        minBlockIncrement = _minBlockIncrement;
    }

    function setMaxRefundAmount(uint256 _maxRefundAmount) external {
        _checkOwner();
        maxRefundAmount = _maxRefundAmount;
    }

    function withdrawFunds(address payable recipient, uint256 amount) external {
        _checkOwner();
        if (recipient == address(0)) {
            revert InvalidAddress();
        }

        (bool success,) = recipient.call{value: amount}("");
        if (!success) {
            revert TransferFailed();
        }

        emit FundsWithdrawn(recipient, amount);
    }

    function transferOwnership(address newOwner) external {
        _checkOwner();
        if (newOwner == address(0)) {
            revert InvalidAddress();
        }
        owner = newOwner;
    }

    /* Upgrade Functions */

    function upgradeTo(address newImplementation, bytes32 expectedCodeHash) external {
        _checkOwner();
        if (newImplementation.code.length == 0) {
            revert InvalidContract();
        }
        if (newImplementation.codehash != expectedCodeHash) {
            revert InvalidCodeHash();
        }

        ERC1967.store(newImplementation);
        emit Upgraded(newImplementation);
    }

    function implementation() external view override returns (address) {
        return ERC1967.load();
    }

    /* View Functions */

    function getCurrentTurnRelayer() external view returns (address) {
        if (relayers.length == 0) {
            return address(0);
        }
        return relayers[currentTurnIndex % relayers.length];
    }

    function isGracePeriodActive() external view returns (bool) {
        return block.number > lastSubmissionBlock + gracePeriodBlocks;
    }

    function getRelayers() external view returns (address[] memory) {
        return relayers;
    }

    function getRelayerCount() external view returns (uint256) {
        return relayers.length;
    }

    function getBalance() external view returns (uint256) {
        return address(this).balance;
    }

    function getCreditedGas(bytes32 commitmentHash) external view returns (uint256) {
        return creditedGas[commitmentHash];
    }

    receive() external payable {
        emit FundsDeposited(msg.sender, msg.value);
    }
}
