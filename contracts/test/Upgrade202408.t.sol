// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.25;

import {Test} from "forge-std/Test.sol";
import {Strings} from "openzeppelin/utils/Strings.sol";
import {console} from "forge-std/console.sol";

import {IGateway} from "../src/interfaces/IGateway.sol";
import {IInitializable} from "../src/interfaces/IInitializable.sol";
import {IUpgradable} from "../src/interfaces/IUpgradable.sol";
import {IGateway} from "../src/interfaces/IGateway.sol";
import {GatewayV2} from "../src/upgrades/GatewayV2.sol";
import {Shell} from "../src/Shell.sol";
import {Upgrade} from "../src/Upgrade.sol";
import {AgentExecutor} from "../src/AgentExecutor.sol";
import {UpgradeParams} from "../src/Params.sol";
import {MockGatewayV2} from "./mocks/MockGatewayV2.sol";
import {Verification} from "../src/Verification.sol";
import {ParaID, InboundMessage, Command, ChannelID} from "../src/Types.sol";

function dot(uint32 value) pure returns (uint128) {
    return value * (10 ** 10);
}

contract Upgrade202408 is Test {
    uint256 mainnetFork;

    // Address of GatewayProxy.sol
    address public constant GATEWAY_ADDR = 0x27ca963C279c93801941e1eB8799c23f407d68e7;

    // Address of Verification.sol library
    address public constant VERIFICATION_ADDR = 0x515c0817005b2F3383B7D8837d6DCc15c0d71C56;

    bytes32[] public proof = [bytes32(0x2f9ee6cfdf244060dc28aa46347c5219e303fc95062dd672b4e406ca5c29764b)];
    bytes public parachainHeaderProof = bytes("validProof");

    function setUp() public {
        mainnetFork = vm.createFork(vm.envString("MAINNET_RPC_URL"));
    }

    function makeMockProof() public pure returns (Verification.Proof memory) {
        return Verification.Proof({
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
    }

    function testUpgrade202408() public {
        vm.selectFork(mainnetFork);

        AgentExecutor executor = new AgentExecutor();

        GatewayV2 impl = new GatewayV2(
            0x6eD05bAa904df3DE117EcFa638d4CB84e1B8A00C,
            address(executor),
            ParaID.wrap(1002),
            0x03170a2e7597b7b7e3d84c05391d139a62b157e78786d8c082f29dcf4c111314,
            10,
            dot(2)
        );

        /// Mock call to Verification.verifyCommitment to bypass BEEFY verification
        vm.mockCall(VERIFICATION_ADDR, abi.encodeWithSelector(Verification.verifyCommitment.selector), abi.encode(true));

        UpgradeParams memory params =
            UpgradeParams({impl: address(impl), implCodeHash: address(impl).codehash, initParams: bytes("")});

        IGateway(GATEWAY_ADDR).submitV1(
            InboundMessage(
                ChannelID.wrap(0x0000000000000000000000000000000000000000000000000000000000000001),
                3,
                Command.Upgrade,
                abi.encode(params),
                500_000,
                1 ether,
                1 ether,
                keccak256("cabbage")
            ),
            proof,
            makeMockProof()
        );
    }
}
