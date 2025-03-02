// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

import {Vm} from "forge-std/Vm.sol";
import {Test} from "forge-std/Test.sol";
import {stdJson} from "forge-std/StdJson.sol";
import {console} from "forge-std/console.sol";

import {IUpgradable} from "../../src/interfaces/IUpgradable.sol";
import {Verification} from "../../src/Verification.sol";
import {UpgradeParams, SetOperatingModeParams, OperatingMode, RegisterForeignTokenParams} from "../../src/Params.sol";
import {ChannelID, ParaID, OperatingMode, InboundMessage, Command, TokenInfo} from "../../src/Types.sol";

struct SubmitMessageFixture {
    InboundMessage message;
    bytes32[] leafProof;
    Verification.Proof headerProof;
}

library ForkTestFixtures {
    using stdJson for string;

    Vm constant public vm = Vm(0x7109709ECfa91a80626fF3989D68f67F5b1DD12D);

    // Make mock proofs for the upgrade message
    function makeMockProofs() internal pure returns (bytes32[] memory, Verification.Proof memory) {
        bytes32[] memory proof1 = new bytes32[](1);
        proof1[0] = bytes32(0x2f9ee6cfdf244060dc28aa46347c5219e303fc95062dd672b4e406ca5c29764b);

        Verification.Proof memory proof2 = Verification.Proof({
            header: Verification.ParachainHeader({
                parentHash: bytes32(0),
                number: 0,
                stateRoot: bytes32(0),
                extrinsicsRoot: bytes32(0),
                digestItems: new Verification.DigestItem[](0)
            }),
            headProof: Verification.HeadProof({pos: 0, width: 0, proof: new bytes32[](0)}),
            leafPartial: Verification.MMRLeafPartial({
                version: 0,
                parentNumber: 0,
                parentHash: bytes32(0),
                nextAuthoritySetID: 0,
                nextAuthoritySetLen: 0,
                nextAuthoritySetRoot: 0
            }),
            leafProof: new bytes32[](0),
            leafProofOrder: 0
        });
        return (proof1, proof2);
    }

    // Create a fixture from real-world mainnet transactions
    function makeSubmitMessageFixture(string memory fixturePath) internal returns (SubmitMessageFixture memory) {
        // Read the test data
        string memory data = vm.readFile(string.concat(vm.projectRoot(), fixturePath));

        // Parse message data
        InboundMessage memory message = InboundMessage({
            channelID: ChannelID.wrap(data.readBytes32(".input.message.channelID")),
            nonce: uint64(data.readUint(".input.message.nonce")),
            command: Command(uint8(data.readUint(".input.message.command"))),
            params: data.readBytes(".input.message.params"),
            maxDispatchGas: uint64(data.readUint(".input.message.maxDispatchGas")),
            maxFeePerGas: data.readUint(".input.message.maxFeePerGas"),
            reward: data.readUint(".input.message.reward"),
            id: data.readBytes32(".input.message.id")
        });

        // Parse proof data
        bytes32[] memory leafProof = new bytes32[](0); // The test data has empty leaf proof

        // Parse header proof
        Verification.ParachainHeader memory header = parseParachainHeader(data);

        Verification.HeadProof memory headProof = Verification.HeadProof({
            pos: uint256(data.readUint(".input.headerProof.headProof.pos")),
            width: uint256(data.readUint(".input.headerProof.headProof.width")),
            proof: data.readBytes32Array(".input.headerProof.headProof.proof")
        });

        Verification.MMRLeafPartial memory leafPartial = Verification.MMRLeafPartial({
            version: uint8(data.readUint(".input.headerProof.leafPartial.version")),
            parentNumber: uint32(data.readUint(".input.headerProof.leafPartial.parentNumber")),
            parentHash: data.readBytes32(".input.headerProof.leafPartial.parentHash"),
            nextAuthoritySetID: uint64(data.readUint(".input.headerProof.leafPartial.nextAuthoritySetID")),
            nextAuthoritySetLen: uint32(data.readUint(".input.headerProof.leafPartial.nextAuthoritySetLen")),
            nextAuthoritySetRoot: data.readBytes32(".input.headerProof.leafPartial.nextAuthoritySetRoot")
        });

        Verification.Proof memory headerProof = Verification.Proof({
            header: header,
            headProof: headProof,
            leafPartial: leafPartial,
            leafProof: data.readBytes32Array(".input.headerProof.leafProof"),
            leafProofOrder: uint256(data.readUint(".input.headerProof.leafProofOrder"))
        });

        SubmitMessageFixture memory fixture = SubmitMessageFixture({
            message: message,
            leafProof: leafProof,
            headerProof: headerProof
        });

        return fixture;
    }

    struct DigestItem {
        bytes consensusEngineID;
        bytes data;
        uint256 kind;
    }

    function parseParachainHeader(string memory data) internal returns (Verification.ParachainHeader memory) {
        bytes32 parentHash = data.readBytes32(".input.headerProof.header.parentHash");
        uint32 number = uint32(data.readUint(".input.headerProof.header.number"));
        bytes32 stateRoot = data.readBytes32(".input.headerProof.header.stateRoot");
        bytes32 extrinsicsRoot = data.readBytes32(".input.headerProof.header.extrinsicsRoot");

        // Parse digest items
        bytes memory digestItems = data.parseRaw(".input.headerProof.header.digestItems");
        DigestItem[] memory items = parseDigestItems(digestItems);

        Verification.DigestItem[] memory finalItems = new Verification.DigestItem[](items.length);
        for (uint256 i = 0; i < items.length; i++) {
            finalItems[i] = Verification.DigestItem({
                kind: items[i].kind,
                consensusEngineID: bytes4(items[i].consensusEngineID),
                data: items[i].data
            });
        }

        return Verification.ParachainHeader({
            parentHash: parentHash,
            number: number,
            stateRoot: stateRoot,
            extrinsicsRoot: extrinsicsRoot,
            digestItems: finalItems
        });
    }

    function parseDigestItems(bytes memory digestItems) internal pure returns (DigestItem[] memory) {
        (DigestItem[] memory items) = abi.decode(digestItems, (DigestItem[]));
        return items;
    }

}
