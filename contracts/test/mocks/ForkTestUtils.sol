// SPDX-License-Identifier: Apache-2.0
import {Verification} from "../../src/Verification.sol";
import {IGateway} from "../../src/interfaces/IGateway.sol";
import {UpgradeParams, RegisterForeignTokenParams} from "../../src/Params.sol";
import {ChannelID, InboundMessage, Command} from "../../src/Types.sol";

library ForkTestUtils {
    ChannelID constant internal PRIMARY_GOVERNANCE_CHANNEL = ChannelID.wrap(0x0000000000000000000000000000000000000000000000000000000000000001);
    ChannelID constant internal SECONDARY_GOVERNANCE_CHANNEL = ChannelID.wrap(0x0000000000000000000000000000000000000000000000000000000000000002);

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

    function upgrade(
        address gateway,
        address impl,
        bytes memory initParams
    ) external {
        UpgradeParams memory params = UpgradeParams({
            impl: address(impl),
            implCodeHash: address(impl).codehash,
            initParams: initParams
        });

        (bytes32[] memory proof1, Verification.Proof memory proof2) = makeMockProofs();
        (uint64 nonce,) = IGateway(gateway).channelNoncesOf(PRIMARY_GOVERNANCE_CHANNEL);

        IGateway(gateway).submitV1(
            InboundMessage(
                PRIMARY_GOVERNANCE_CHANNEL,
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
    }

    function registerForeignToken(
        address gateway,
        bytes32 foreignTokenID,
        string memory name,
        string memory symbol,
        uint8 decimals
    ) external {
        RegisterForeignTokenParams memory params =
            RegisterForeignTokenParams({
                foreignTokenID: foreignTokenID,
                name: name,
                symbol: symbol,
                decimals: decimals
            });

        (bytes32[] memory proof1, Verification.Proof memory proof2) = makeMockProofs();
        (uint64 nonce,) = IGateway(gateway).channelNoncesOf(SECONDARY_GOVERNANCE_CHANNEL);

        IGateway(gateway).submitV1(
            InboundMessage(
                SECONDARY_GOVERNANCE_CHANNEL,
                nonce + 1,
                Command.RegisterForeignToken,
                abi.encode(params),
                1_200_000,
                block.basefee,
                0,
                keccak256("message-id")
            ),
            proof1,
            proof2
        );
    }
}
