pragma solidity ^0.8.9;

import "../../BeefyClient.sol";
import "@openzeppelin/contracts/utils/Strings.sol";
import "forge-std/Test.sol";
import "forge-std/console.sol";

contract BeefyClientTest is Test {
    BeefyClient beefyClient;
    uint32 setSize;
    uint32 setId;
    address validator;
    uint8 v;
    bytes32 r;
    bytes32 s;
    bytes32 commitHash;
    bytes32 root;
    uint256[] bitSetArray;
    uint256[] bitfield;
    bytes32[] proofs;

    function setUp() public {
        beefyClient = new BeefyClient(3, 8);
        setSize = 300;
        setId = 37;
        commitHash = 0x243baf0066d021d42716081dad0b30499dad95a300daa269ed8f6f6334d95975;
        string[] memory inputs = new string[](5);
        inputs[0] = "test/beefy/validator-set.ts";
        inputs[1] = Strings.toString(setId);
        inputs[2] = Strings.toString(setSize);
        inputs[3] = Strings.toHexString(uint256(commitHash), 32);
        (root, proofs, bitSetArray, validator, v, r, s) = abi.decode(
            vm.ffi(inputs),
            (bytes32, bytes32[], uint256[], address, uint8, bytes32, bytes32)
        );
        console.logBytes32(root);
        console.logBytes32(proofs[0]);
        console.logUint(bitSetArray[0]);
        console.logAddress(validator);
        console.logUint(v);
        console.logBytes32(r);
        console.logBytes32(s);

        BeefyClient.ValidatorSet memory vset = BeefyClient.ValidatorSet(
            uint128(bitSetArray[0]),
            setSize,
            root
        );
        BeefyClient.ValidatorSet memory nextvset = BeefyClient.ValidatorSet(
            setId + 1,
            setSize,
            root
        );
        beefyClient.initialize(0, vset, nextvset);
    }

    function testSubmitInitial() public {
        bitfield = beefyClient.createInitialBitfield(bitSetArray, setSize);
        BeefyClient.ValidatorProof memory validateProof = BeefyClient.ValidatorProof(
            v,
            r,
            s,
            bitSetArray[0],
            validator,
            proofs
        );
        beefyClient.submitInitial(commitHash, bitfield, validateProof);
    }
}
