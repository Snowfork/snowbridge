// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.5;
pragma experimental ABIEncoderV2;

import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/utils/cryptography/ECDSA.sol";

contract ValidatorRegistryFull is Ownable {
    /* Events */

    event AddressesUpdated(address[] addresses, uint64 id);

    /* State */

    address[] addresses;
    uint64 public id;

    constructor(address[] memory _addresses, uint64 _id) {
        addresses = _addresses;
        id = _id;
    }

    function numOfValidators() public view returns (uint256) {
        return addresses.length;
    }

    function update(address[] calldata _addresses, uint64 _id)
        public
        onlyOwner
    {
        addresses = _addresses;
        id = _id;
        emit AddressesUpdated(_addresses, _id);
    }

    function checkSignatures(
        bytes[] calldata signatures,
        bytes32 commitmentHash
    ) public view returns (uint256) {
        uint256 correctSignatures = 0;
        require(
            signatures.length == addresses.length,
            "Signature count does not match address count"
        );
        for (uint256 i = 0; i < addresses.length; i++) {
            address recoveredAddress = ECDSA.recover(
                commitmentHash,
                signatures[i]
            );
            if (recoveredAddress == addresses[i]) {
                correctSignatures++;
            }
        }
        return correctSignatures;
    }
}
