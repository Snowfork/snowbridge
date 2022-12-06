pragma solidity ^0.8.9;

import "../../BeefyClient.sol";
import "@openzeppelin/contracts/utils/Strings.sol";
import "forge-std/Test.sol";
import "forge-std/console.sol";


contract BeefyClientTest is Test {
    BeefyClient beefyClient;
    uint32 setSize;
    uint32 setId;
    uint32 setIndex;
    address validator;
    uint8 v;
    bytes32 r;
    bytes32 s;
    bytes32 commitHash;
    bytes32 root;
    uint256[] bitSetArray;
    uint256[] bitfield;
    bytes32[] proofs;
    
    event NewRequest(uint256 id, address sender);

    function setUp() public {
        beefyClient = new BeefyClient();
        setSize = 10;
        setId = 36;
        setIndex = 1;
        commitHash = 0x243baf0066d021d42716081dad0b30499dad95a300daa269ed8f6f6334d95975;
        
        string[] memory inputs = new string[](5);
        inputs[0] = "test/beefy/validator-set.ts";
        inputs[1] = Strings.toString(setId);
        inputs[2] = Strings.toString(setSize);
        inputs[3] = Strings.toString(setIndex);
        inputs[4] = Strings.toHexString(uint256(commitHash), 32);
        (root,proofs,bitSetArray,validator,v,r,s) = abi.decode(
            vm.ffi(inputs),
            (bytes32,bytes32[],uint256[],address,uint8,bytes32,bytes32)
        );
        console.logBytes32(root);
        console.logBytes32(proofs[0]);
        console.logUint(bitSetArray[0]);
        console.logAddress(validator);
        console.logUint(v);
        console.logBytes32(r);
        console.logBytes32(s);
    }

    function testSubmitInitial() public {
        bitfield = beefyClient.createInitialBitfield(bitSetArray, setSize);
        BeefyClient.ValidatorSet memory vset = BeefyClient.ValidatorSet(setId,root,setSize);
        BeefyClient.ValidatorSet memory nextvset = BeefyClient.ValidatorSet(setId+1,root,setSize);
        beefyClient.initialize(0,vset,nextvset);
        BeefyClient.ValidatorSignature memory signature = BeefyClient.ValidatorSignature(v,r,s);
        BeefyClient.ValidatorProof memory validateProof = BeefyClient.ValidatorProof(signature,setIndex,validator,proofs);
        vm.expectEmit(false, false,false, true);
        emit NewRequest(0, address(this));
        beefyClient.submitInitial(
            commitHash,
            setId,
            bitfield,
            validateProof
       );
       assertEq(beefyClient.nextRequestID(),1);
    }
}