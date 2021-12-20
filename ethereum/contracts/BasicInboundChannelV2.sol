// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.5;
pragma experimental ABIEncoderV2;
import "./ParachainLightClient.sol";
import "./BeefyLightClient.sol";
import "./SimplifiedMMRVerification.sol";
import "./utils/MerkleProof.sol";

contract BasicInboundChannelV2 {
    uint256 public constant MAX_GAS_PER_MESSAGE = 100000;
    uint256 public constant GAS_BUFFER = 60000;

    mapping(bytes32 => uint64) public userNonce;

    BeefyLightClient public beefyLightClient;

    struct Leaf {
        bytes32 account;
        uint64 nonce;
        Message[] messages;
    }

    struct Message {
        address target;
        bytes payload;
    }

    event MessageDispatched(uint64 nonce, bool result);

    constructor(BeefyLightClient _beefyLightClient) {
        beefyLightClient = _beefyLightClient;
    }

    function generateCommitmentHash(Leaf calldata _leaf,bytes32[] calldata _leafProof,bool[] calldata nodeSide)
    internal pure returns (bytes32)
    {
        bytes32 leafNodeHash = keccak256(abi.encode(_leaf));

        return MerkleProof.computeRootFromProofAndSide(leafNodeHash,_leafProof,nodeSide);
    }

    function submit(
       Leaf calldata _leaf,
       bytes32[] calldata leafProof,
        bool[] calldata nodeSide,
        ParachainLightClient.ParachainVerifyInput
            calldata _parachainVerifyInput,
        ParachainLightClient.BeefyMMRLeafPartial calldata _beefyMMRLeafPartial,
        SimplifiedMMRProof calldata proof
    ) public {

        bytes32 commitment = generateCommitmentHash(_leaf,leafProof,nodeSide);

        ParachainLightClient.verifyCommitmentInParachain(
            commitment,
            _parachainVerifyInput,
            _beefyMMRLeafPartial,
            proof,
            beefyLightClient
        );

        // Require there is enough gas to play all messages
        require(
            gasleft() >= (_leaf.messages.length * MAX_GAS_PER_MESSAGE) + GAS_BUFFER,
            "insufficient gas for delivery of all messages"
        );

        processMessages(_leaf);
    }

    function processMessages(Leaf calldata _leaf) public { 
        // User nonce for replay protection
        uint64 usercachedNonce =  userNonce[_leaf.account];
        
        require(usercachedNonce + 1  == _leaf.nonce , "invalid nonce");

         for (uint256 i = usercachedNonce; i <  _leaf.messages.length + userNonce[_leaf.account]; i++) {
            
            usercachedNonce = usercachedNonce +1;
            
            // Deliver the message to the target
            (bool success, ) = _leaf.messages[i].target.call{
                value: 0,
                gas: MAX_GAS_PER_MESSAGE
            }(_leaf.messages[i].payload);

            emit MessageDispatched(usercachedNonce, success);
        }
        userNonce[_leaf.account] = usercachedNonce;
    }
}