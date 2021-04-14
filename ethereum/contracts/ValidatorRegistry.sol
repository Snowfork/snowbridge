// SPDX-License-Identifier: Apache-2.0
pragma solidity >=0.7.6;
pragma experimental ABIEncoderV2;

import "@openzeppelin/contracts/access/Ownable.sol";
import "./utils/MerkleProof.sol";

/**
 * @title A contract storing state on the current validator set
 * @dev Stores the validator set as a Merkle root
 * @dev Inherits `Ownable` to ensure it can only be callable by the
 * instantiating contract account (which is the LightClientBridge contract)
 */
contract ValidatorRegistry is Ownable {
    /* Events */

    event ValidatorRegistryUpdated(bytes32 root, uint256 numOfValidators);

    /* State */

    bytes32 public root;
    uint256 public numOfValidators;

    constructor(bytes32 _root, uint256 _numOfValidators) {
        root = _root;
        numOfValidators = _numOfValidators;
    }

    /**
     * @notice Updates the validator registry and number of validators
     * @param _root The new root
     * @param _numOfValidators The new number of validators
     */
    function update(bytes32 _root, uint256 _numOfValidators) public onlyOwner {
        root = _root;
        numOfValidators = _numOfValidators;
        emit ValidatorRegistryUpdated(_root, _numOfValidators);
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
                numOfValidators,
                proof
            );
    }
}
