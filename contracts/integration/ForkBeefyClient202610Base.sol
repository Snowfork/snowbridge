// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.34;

import {Test} from "forge-std/Test.sol";

import {AgentExecutor} from "../src/AgentExecutor.sol";
import {BeefyClient} from "../src/BeefyClient.sol";
import {Gateway} from "../src/Gateway.sol";
import {IUpgradable} from "../src/interfaces/IUpgradable.sol";
import {Gateway202610} from "../src/upgrade/Gateway202610.sol";
import {IGatewayV1} from "../src/v1/IGateway.sol";
import {ChannelID, Command, InboundMessage, UpgradeParams} from "../src/v1/Types.sol";
import {Verification} from "../src/Verification.sol";
import {ForkTestFixtures} from "../test/utils/ForkTestFixtures.sol";

/// @notice Replays mainnet BEEFY transactions against a BeefyClient built from this branch,
///         checkpointed from the production client's state at the forked transaction.
abstract contract ForkBeefyClient202610Base is Test {
    address internal constant GATEWAY_PROXY = 0x27ca963C279c93801941e1eB8799c23f407d68e7;
    address internal constant OLD_BEEFY_CLIENT = 0x7cfc5C8b341991993080Af67D940B6aD19a010E1;
    // Verification library linked into the Gateway logic live at the replayed blocks.
    address internal constant VERIFICATION = 0x53fe58beFD174f513871519655c120436C128152;
    ChannelID internal constant GOVERNANCE_CHANNEL =
        ChannelID.wrap(0x0000000000000000000000000000000000000000000000000000000000000001);
    address internal constant RELAYER = 0xBa9bC9a8Aa87872f7B990031bde984A00b9CEd49;

    // DeployBeefyClient.mainnetConfig
    uint256 internal constant RANDAO_COMMIT_DELAY = 128;
    uint256 internal constant RANDAO_COMMIT_EXPIRATION = 12;
    uint256 internal constant MINIMUM_SIGNATURES = 20;
    uint256 internal constant FIAT_SHAMIR_REQUIRED_SIGNATURES = 103;

    struct FinalArgs {
        BeefyClient.Commitment commitment;
        uint256[] bitfield;
        BeefyClient.ValidatorProof[] proofs;
        BeefyClient.MMRLeaf leaf;
        bytes32[] leafProof;
        uint256 leafProofOrder;
    }

    /// @dev Forks at the end of the previous block. No earlier transaction in the replayed
    ///      blocks calls the BeefyClient, the Gateway or the Multicall3 directly.
    function forkAt(uint256 blockNumber) internal {
        vm.createSelectFork(vm.envString("MAINNET_ARCHIVE_RPC_URL"), blockNumber - 1);
        vm.roll(blockNumber);
    }

    function deployCheckpointedClient() internal returns (BeefyClient client) {
        BeefyClient old = BeefyClient(OLD_BEEFY_CLIENT);
        (uint128 currentId, uint128 currentLength, bytes32 currentRoot,) =
            old.currentValidatorSet();
        (uint128 nextId, uint128 nextLength, bytes32 nextRoot,) = old.nextValidatorSet();
        client = new BeefyClient(
            RANDAO_COMMIT_DELAY,
            RANDAO_COMMIT_EXPIRATION,
            MINIMUM_SIGNATURES,
            FIAT_SHAMIR_REQUIRED_SIGNATURES,
            old.latestBeefyBlock(),
            BeefyClient.ValidatorSet(currentId, currentLength, currentRoot),
            BeefyClient.ValidatorSet(nextId, nextLength, nextRoot)
        );
    }

    /// @dev Points the production Gateway at `beefyClient`, as `ForkUpgrade202610` does.
    function upgradeGateway(address beefyClient) internal {
        vm.mockCall(VERIFICATION, bytes(""), abi.encode(true));

        AgentExecutor executor = new AgentExecutor();
        Gateway202610 newLogic = new Gateway202610(beefyClient, address(executor));
        UpgradeParams memory params = UpgradeParams({
            impl: address(newLogic), implCodeHash: address(newLogic).codehash, initParams: ""
        });
        (bytes32[] memory proof1, Verification.Proof memory proof2) =
            ForkTestFixtures.makeMockProofs();
        (uint64 nonce,) = IGatewayV1(GATEWAY_PROXY).channelNoncesOf(GOVERNANCE_CHANNEL);

        vm.expectEmit();
        emit IUpgradable.Upgraded(address(newLogic));
        IGatewayV1(GATEWAY_PROXY)
            .submitV1(
                InboundMessage(
                    GOVERNANCE_CHANNEL,
                    nonce + 1,
                    Command.Upgrade,
                    abi.encode(params),
                    100_000,
                    block.basefee,
                    0,
                    keccak256("message-id")
                ),
                proof1,
                proof2
            );

        vm.clearMockedCalls();
        assertEq(Gateway(payable(GATEWAY_PROXY)).BEEFY_CLIENT(), beefyClient);
    }

    function readFixture(string memory name) internal view returns (bytes memory) {
        return vm.parseBytes(
            vm.trim(vm.readFile(string.concat(vm.projectRoot(), "/integration/data/", name)))
        );
    }

    function decodeFinalArgs(bytes memory args) internal pure returns (FinalArgs memory f) {
        (f.commitment, f.bitfield, f.proofs, f.leaf, f.leafProof, f.leafProofOrder) = abi.decode(
            args,
            (
                BeefyClient.Commitment,
                uint256[],
                BeefyClient.ValidatorProof[],
                BeefyClient.MMRLeaf,
                bytes32[],
                uint256
            )
        );
    }

    function stripSelector(bytes memory data) internal pure returns (bytes memory out) {
        out = new bytes(data.length - 4);
        for (uint256 i = 0; i < out.length; i++) {
            out[i] = data[i + 4];
        }
    }

    function isSet(uint256[] memory bits, uint256 index) internal pure returns (bool) {
        return (bits[index / 256] >> (index % 256)) & 1 == 1;
    }

    function countSet(uint256[] memory bits) internal pure returns (uint256 n) {
        for (uint256 i = 0; i < bits.length; i++) {
            for (uint256 w = bits[i]; w != 0; w &= w - 1) {
                n++;
            }
        }
    }

    /// @dev Keeps the proofs whose validator index is set in `bits`.
    function selectProofs(BeefyClient.ValidatorProof[] memory pool, uint256[] memory bits)
        internal
        pure
        returns (BeefyClient.ValidatorProof[] memory out)
    {
        out = new BeefyClient.ValidatorProof[](pool.length);
        uint256 n;
        for (uint256 i = 0; i < pool.length; i++) {
            if (isSet(bits, pool[i].index)) {
                out[n++] = pool[i];
            }
        }
        assembly {
            mstore(out, n)
        }
    }

    function ticketID(address relayer, bytes32 commitmentHash) internal pure returns (bytes32) {
        return keccak256(abi.encode(relayer, commitmentHash));
    }
}
