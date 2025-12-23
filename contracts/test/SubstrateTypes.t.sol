// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

import {Test} from "forge-std/Test.sol";
import {SubstrateTypes} from "../src/SubstrateTypes.sol";
import {ScaleCodec} from "../src/utils/ScaleCodec.sol";
import {ParaID} from "../src/v1/Types.sol";

contract SubstrateTypesWrapper {
    function H160(address account) external pure returns (bytes memory) {
        return SubstrateTypes.H160(account);
    }

    function VecU8(bytes calldata input) external pure returns (bytes memory) {
        return SubstrateTypes.VecU8(input);
    }

    function None() external pure returns (bytes memory) {
        return SubstrateTypes.None();
    }

    function RegisterToken(address token, uint128 fee) external view returns (bytes memory) {
        return SubstrateTypes.RegisterToken(token, fee);
    }

    function SendTokenToAssetHubAddress32(
        address token,
        bytes32 recipient,
        uint128 xcmFee,
        uint128 amount
    ) external view returns (bytes memory) {
        return SubstrateTypes.SendTokenToAssetHubAddress32(token, recipient, xcmFee, amount);
    }

    function SendTokenToAddress32(
        address token,
        uint32 para,
        bytes32 recipient,
        uint128 xcmFee,
        uint128 destinationXcmFee,
        uint128 amount
    ) external view returns (bytes memory) {
        return SubstrateTypes.SendTokenToAddress32(
            token, ParaID.wrap(para), recipient, xcmFee, destinationXcmFee, amount
        );
    }

    function SendTokenToAddress20(
        address token,
        uint32 para,
        bytes20 recipient,
        uint128 xcmFee,
        uint128 destinationXcmFee,
        uint128 amount
    ) external view returns (bytes memory) {
        return SubstrateTypes.SendTokenToAddress20(
            token, ParaID.wrap(para), recipient, xcmFee, destinationXcmFee, amount
        );
    }

    function SendForeignTokenToAssetHubAddress32(
        bytes32 tokenID,
        bytes32 recipient,
        uint128 xcmFee,
        uint128 amount
    ) external view returns (bytes memory) {
        return SubstrateTypes.SendForeignTokenToAssetHubAddress32(
            tokenID, recipient, xcmFee, amount
        );
    }

    function SendForeignTokenToAddress32(
        bytes32 tokenID,
        uint32 para,
        bytes32 recipient,
        uint128 xcmFee,
        uint128 destinationXcmFee,
        uint128 amount
    ) external view returns (bytes memory) {
        return SubstrateTypes.SendForeignTokenToAddress32(
            tokenID, ParaID.wrap(para), recipient, xcmFee, destinationXcmFee, amount
        );
    }

    function SendForeignTokenToAddress20(
        bytes32 tokenID,
        uint32 para,
        bytes20 recipient,
        uint128 xcmFee,
        uint128 destinationXcmFee,
        uint128 amount
    ) external view returns (bytes memory) {
        return SubstrateTypes.SendForeignTokenToAddress20(
            tokenID, ParaID.wrap(para), recipient, xcmFee, destinationXcmFee, amount
        );
    }
}

contract SubstrateTypesTest is Test {
    SubstrateTypesWrapper public wrapper;

    function setUp() public {
        wrapper = new SubstrateTypesWrapper();
    }

    // Use authoritative encoders from `ScaleCodec` to build expected bytes

    // --- Tests ---
    function testH160() public {
        address a = address(0x1234567890AbcdEF1234567890aBcdef12345678);
        bytes memory got = wrapper.H160(a);
        bytes memory expect = abi.encodePacked(a);
        assertEq(keccak256(got), keccak256(expect));
    }

    function testVecU8() public {
        bytes memory data = hex"1122334455";
        bytes memory got = wrapper.VecU8(data);
        bytes memory prefix = ScaleCodec.checkedEncodeCompactU32(data.length);
        bytes memory expect = bytes.concat(prefix, data);
        assertEq(keccak256(got), keccak256(expect));
    }

    function testNone() public {
        bytes memory got = wrapper.None();
        assertEq(got.length, 1);
        assertEq(uint8(got[0]), 0);
    }

    function testRegisterToken() public {
        address token = address(0xdEADBEeF00000000000000000000000000000000);
        uint128 fee = 123_456;
        bytes memory got = wrapper.RegisterToken(token, fee);

        bytes memory expect = bytes.concat(
            bytes1(0x00),
            abi.encodePacked(ScaleCodec.encodeU64(uint64(block.chainid))),
            bytes1(0x00),
            abi.encodePacked(token),
            abi.encodePacked(ScaleCodec.encodeU128(fee))
        );

        assertEq(keccak256(got), keccak256(expect));
    }

    function testSendTokenToAssetHubAddress32() public {
        address token = address(0x1111111111222222222233333333333344444444);
        bytes32 recipient = keccak256("recipient32");
        uint128 xcmFee = 10;
        uint128 amount = 1000;

        bytes memory got = wrapper.SendTokenToAssetHubAddress32(token, recipient, xcmFee, amount);

        bytes memory expect = bytes.concat(
            bytes1(0x00),
            abi.encodePacked(ScaleCodec.encodeU64(uint64(block.chainid))),
            bytes1(0x01),
            abi.encodePacked(token),
            bytes1(0x00),
            recipient,
            abi.encodePacked(ScaleCodec.encodeU128(amount)),
            abi.encodePacked(ScaleCodec.encodeU128(xcmFee))
        );

        assertEq(keccak256(got), keccak256(expect));
    }

    function testSendTokenToAddress32() public {
        address token = address(0x2222000000000000000000000000000000000000);
        uint32 para = 42;
        bytes32 recipient = keccak256("r32");
        uint128 xcmFee = 7;
        uint128 destFee = 9;
        uint128 amount = 555;

        bytes memory got =
            wrapper.SendTokenToAddress32(token, para, recipient, xcmFee, destFee, amount);

        bytes memory expect = bytes.concat(
            bytes1(0x00),
            abi.encodePacked(ScaleCodec.encodeU64(uint64(block.chainid))),
            bytes1(0x01),
            abi.encodePacked(token),
            bytes1(0x01),
            abi.encodePacked(ScaleCodec.encodeU32(para)),
            recipient,
            abi.encodePacked(ScaleCodec.encodeU128(destFee)),
            abi.encodePacked(ScaleCodec.encodeU128(amount)),
            abi.encodePacked(ScaleCodec.encodeU128(xcmFee))
        );

        assertEq(keccak256(got), keccak256(expect));
    }

    function testSendTokenToAddress20() public {
        address token = address(0x3333000000000000000000000000000000000000);
        uint32 para = 7;
        bytes20 recipient = bytes20(address(0xABcdEFABcdEFabcdEfAbCdefabcdeFABcDEFabCD));
        uint128 xcmFee = 1;
        uint128 destFee = 2;
        uint128 amount = 3;

        bytes memory got =
            wrapper.SendTokenToAddress20(token, para, recipient, xcmFee, destFee, amount);

        bytes memory expect = bytes.concat(
            bytes1(0x00),
            abi.encodePacked(ScaleCodec.encodeU64(uint64(block.chainid))),
            bytes1(0x01),
            abi.encodePacked(token),
            bytes1(0x02),
            abi.encodePacked(ScaleCodec.encodeU32(para)),
            abi.encodePacked(recipient),
            abi.encodePacked(ScaleCodec.encodeU128(destFee)),
            abi.encodePacked(ScaleCodec.encodeU128(amount)),
            abi.encodePacked(ScaleCodec.encodeU128(xcmFee))
        );

        assertEq(keccak256(got), keccak256(expect));
    }

    function testSendForeignTokenToAssetHubAddress32() public {
        bytes32 tokenID = keccak256("tokenid");
        bytes32 recipient = keccak256("r");
        uint128 xcmFee = 4;
        uint128 amount = 400;

        bytes memory got =
            wrapper.SendForeignTokenToAssetHubAddress32(tokenID, recipient, xcmFee, amount);

        bytes memory expect = bytes.concat(
            bytes1(0x00),
            abi.encodePacked(ScaleCodec.encodeU64(uint64(block.chainid))),
            bytes1(0x02),
            tokenID,
            bytes1(0x00),
            recipient,
            abi.encodePacked(ScaleCodec.encodeU128(amount)),
            abi.encodePacked(ScaleCodec.encodeU128(xcmFee))
        );

        assertEq(keccak256(got), keccak256(expect));
    }

    function testSendForeignTokenToAddress32() public {
        bytes32 tokenID = keccak256("t2");
        uint32 para = 3;
        bytes32 recipient = keccak256("r2");
        uint128 xcmFee = 6;
        uint128 destFee = 11;
        uint128 amount = 9999;

        bytes memory got =
            wrapper.SendForeignTokenToAddress32(tokenID, para, recipient, xcmFee, destFee, amount);

        bytes memory expect = bytes.concat(
            bytes1(0x00),
            abi.encodePacked(ScaleCodec.encodeU64(uint64(block.chainid))),
            bytes1(0x02),
            tokenID,
            bytes1(0x01),
            abi.encodePacked(ScaleCodec.encodeU32(para)),
            recipient,
            abi.encodePacked(ScaleCodec.encodeU128(destFee)),
            abi.encodePacked(ScaleCodec.encodeU128(amount)),
            abi.encodePacked(ScaleCodec.encodeU128(xcmFee))
        );

        assertEq(keccak256(got), keccak256(expect));
    }

    function testSendForeignTokenToAddress20() public {
        bytes32 tokenID = keccak256("t3");
        uint32 para = 99;
        bytes20 recipient = bytes20(address(0x3333333333333333333333333333333333333333));
        uint128 xcmFee = 12;
        uint128 destFee = 13;
        uint128 amount = 14;

        bytes memory got =
            wrapper.SendForeignTokenToAddress20(tokenID, para, recipient, xcmFee, destFee, amount);

        bytes memory expect = bytes.concat(
            bytes1(0x00),
            abi.encodePacked(ScaleCodec.encodeU64(uint64(block.chainid))),
            bytes1(0x02),
            tokenID,
            bytes1(0x02),
            abi.encodePacked(ScaleCodec.encodeU32(para)),
            abi.encodePacked(recipient),
            abi.encodePacked(ScaleCodec.encodeU128(destFee)),
            abi.encodePacked(ScaleCodec.encodeU128(amount)),
            abi.encodePacked(ScaleCodec.encodeU128(xcmFee))
        );

        assertEq(keccak256(got), keccak256(expect));
    }
}
