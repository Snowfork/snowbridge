// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.34;

import {BeefyClient} from "../src/BeefyClient.sol";
import {BeefyClientTest} from "./BeefyClient.t.sol";
import {CompactProofLib} from "./utils/CompactProofLib.sol";

// Gas with latestMMRRoot / latestBeefyBlock already non-zero, as on mainnet.
contract BeefyClientWarmGasTest is BeefyClientTest {
    function warm() internal returns (BeefyClient.Commitment memory commitment) {
        commitment = initialize(setId);
        beefyClient.setLatestMMRRoot(bytes32(uint256(1)));
        beefyClient.setLatestBeefyBlock(1);
    }

    function testWarmInteractive() public {
        BeefyClient.Commitment memory commitment = warm();
        beefyClient.submitInitial(commitment, bitfield, finalValidatorProofs[0]);
        vm.roll(block.number + randaoCommitDelay);
        commitPrevRandao();
        createFinalProofs();
        beefyClient.submitFinal(
            commitment,
            bitfield,
            CompactProofLib.toCompact(finalValidatorProofs, setSize),
            emptyLeaf,
            emptyLeafProofs,
            emptyLeafProofOrder
        );
    }

    function testWarmFiatShamir() public {
        BeefyClient.Commitment memory commitment = warm();
        beefyClient.submitFiatShamir(
            commitment,
            bitfield,
            CompactProofLib.toCompact(fiatShamirValidatorProofs, setSize),
            emptyLeaf,
            emptyLeafProofs,
            emptyLeafProofOrder
        );
    }

    function testCalldataSize() public {
        BeefyClient.Commitment memory commitment = initialize(setId);
        emit log_named_uint(
            "submitFinal calldata",
            abi.encodeCall(
                BeefyClient.submitFinal,
                (
                    commitment,
                    bitfield,
                    CompactProofLib.toCompact(finalValidatorProofs, setSize),
                    emptyLeaf,
                    emptyLeafProofs,
                    emptyLeafProofOrder
                )
            )
            .length
        );
        emit log_named_uint(
            "submitFiatShamir calldata",
            abi.encodeCall(
                BeefyClient.submitFiatShamir,
                (
                    commitment,
                    bitfield,
                    CompactProofLib.toCompact(fiatShamirValidatorProofs, setSize),
                    emptyLeaf,
                    emptyLeafProofs,
                    emptyLeafProofOrder
                )
            )
            .length
        );
    }
}
