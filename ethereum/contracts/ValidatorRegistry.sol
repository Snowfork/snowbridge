// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.5;
pragma experimental ABIEncoderV2;

import "@openzeppelin/contracts/access/Ownable.sol";
import "./utils/MerkleProof.sol";

/**
 * @title A contract storing state on the current validator set
 * @dev Stores the validator set as a Merkle root
 * @dev Inherits `Ownable` to ensure it can only be callable by the
 * instantiating contract account (which is the BeefyLightClient contract)
 */
contract ValidatorRegistry is Ownable {
    /* Events */

    event ValidatorRegistryUpdated(
        bytes32 root,
        uint256 numOfValidators,
        uint64 id
    );

    /* State */

    bytes32 public root;
    uint256 public numOfValidators;
    uint64 public id;

    /**
     * @notice Updates the validator registry and number of validators
     * @param _root The new root
     * @param _numOfValidators The new number of validators
     */
    function update(
        bytes32 _root,
        uint256 _numOfValidators,
        uint64 _id
    ) public onlyOwner {
        root = _root;
        numOfValidators = _numOfValidators;
        id = _id;
        emit ValidatorRegistryUpdated(_root, _numOfValidators, _id);
    }

    /**
     * @notice Checks if a validators address is a member of the merkle tree
     * @param addr The address of the validator to check
     * @param pos The position of the validator to check, index starting at 0
     * @param proof Merkle proof required for validation of the address
     * @return Returns true if the validator is in the set
     */
    function checkValidatorInSet(
        address addr,
        uint256 pos,
        bytes32[] memory proof
    ) public view returns (bool) {
        bytes32 hashedLeaf = keccak256(abi.encodePacked(addr));
        return
            MerkleProof.verifyMerkleLeafAtPosition(
                root,
                hashedLeaf,
                pos,
                roundUpToPow2(numOfValidators),
                proof
            );
    }

    function roundUpToPow2(uint256 len) internal pure returns (uint256) {
        if (len <= 1) return 1;
        else return 2 * roundUpToPow2((len + 1) / 2);
    }
}
