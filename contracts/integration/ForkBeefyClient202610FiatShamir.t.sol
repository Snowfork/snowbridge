// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.34;

import {Vm} from "forge-std/Vm.sol";

import {BeefyClient} from "../src/BeefyClient.sol";
import {IGatewayV1} from "../src/v1/IGateway.sol";
import {IGatewayV2} from "../src/v2/IGateway.sol";
import {ForkBeefyClient202610Base} from "./ForkBeefyClient202610Base.sol";

interface IMulticall3 {
    struct Call3 {
        address target;
        bool allowFailure;
        bytes callData;
    }

    struct Result {
        bool success;
        bytes returnData;
    }

    function aggregate3(Call3[] calldata calls) external payable returns (Result[] memory);
}

/// @notice Fiat-Shamir submissions from mainnet, replayed against the new BeefyClient.
contract ForkBeefyClient202610FiatShamirTest is ForkBeefyClient202610Base {
    // Tx 0x9bcfd377…: a sole submitFiatShamir.
    uint256 private constant FS_BLOCK = 26_276_559;
    // Tx 0xa9b70536…: Multicall3 aggregate3 of submitFiatShamir, 4 x v2_submit and submitV1.
    uint256 private constant AGGREGATE_BLOCK = 26_276_567;
    address private constant MULTICALL = 0xF57F9e622bc12DA707dB20a9e21Cd88F212efFC1;

    /// @dev The sample is a prefix of the original 111, so the original proofs cover it.
    function resample(BeefyClient client, FinalArgs memory f)
        private
        view
        returns (BeefyClient.ValidatorProof[] memory proofs)
    {
        uint256[] memory finalBitfield =
            client.createFiatShamirFinalBitfield(f.commitment, f.bitfield);
        assertEq(countSet(finalBitfield), FIAT_SHAMIR_REQUIRED_SIGNATURES);
        proofs = selectProofs(f.proofs, finalBitfield);
        assertEq(proofs.length, FIAT_SHAMIR_REQUIRED_SIGNATURES);
    }

    function testSubmitFiatShamir() public {
        forkAt(FS_BLOCK);
        BeefyClient client = deployCheckpointedClient();
        FinalArgs memory f = decodeFinalArgs(readFixture("tx-fs-submitFiatShamir.hex"));
        assertEq(f.proofs.length, 111);

        vm.prank(RELAYER);
        vm.expectRevert(BeefyClient.InvalidValidatorProofLength.selector);
        client.submitFiatShamir(
            f.commitment, f.bitfield, f.proofs, f.leaf, f.leafProof, f.leafProofOrder
        );

        BeefyClient.ValidatorProof[] memory proofs = resample(client, f);
        vm.prank(RELAYER);
        uint256 gasBefore = gasleft();
        client.submitFiatShamir(
            f.commitment, f.bitfield, proofs, f.leaf, f.leafProof, f.leafProofOrder
        );
        emit log_named_uint("submitFiatShamir gas (103 proofs)", gasBefore - gasleft());

        assertEq(client.latestBeefyBlock(), f.commitment.blockNumber);
    }

    function testAggregate3WithMessages() public {
        forkAt(AGGREGATE_BLOCK);
        BeefyClient client = deployCheckpointedClient();
        upgradeGateway(address(client));

        IMulticall3.Call3[] memory calls =
            abi.decode(readFixture("tx-agg-aggregate3.hex"), (IMulticall3.Call3[]));
        assertEq(calls.length, 6);
        assertEq(calls[0].target, OLD_BEEFY_CLIENT);
        assertEq(bytes4(calls[0].callData), BeefyClient.submitFiatShamir.selector);

        FinalArgs memory f = decodeFinalArgs(stripSelector(calls[0].callData));
        calls[0].target = address(client);
        calls[0].callData = abi.encodeCall(
            BeefyClient.submitFiatShamir,
            (f.commitment, f.bitfield, resample(client, f), f.leaf, f.leafProof, f.leafProofOrder)
        );

        vm.recordLogs();
        vm.prank(RELAYER);
        IMulticall3(MULTICALL).aggregate3(calls);
        Vm.Log[] memory logs = vm.getRecordedLogs();

        assertEq(client.latestBeefyBlock(), f.commitment.blockNumber);

        uint256 v1Dispatched;
        uint256 v2Dispatched;
        for (uint256 i = 0; i < logs.length; i++) {
            if (logs[i].emitter != GATEWAY_PROXY) continue;
            if (logs[i].topics[0] == IGatewayV2.InboundMessageDispatched.selector) {
                (, bool success,) = abi.decode(logs[i].data, (bytes32, bool, bytes32));
                assertTrue(success);
                assertTrue(
                    IGatewayV2(GATEWAY_PROXY).v2_isDispatched(uint64(uint256(logs[i].topics[1])))
                );
                v2Dispatched++;
            } else if (logs[i].topics[0] == IGatewayV1.InboundMessageDispatched.selector) {
                (, bool success) = abi.decode(logs[i].data, (uint64, bool));
                assertTrue(success);
                v1Dispatched++;
            }
        }
        assertEq(v2Dispatched, 4);
        assertEq(v1Dispatched, 1);
    }
}
