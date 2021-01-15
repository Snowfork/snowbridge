// SPDX-License-Identifier: MIT
pragma solidity >=0.6.2;
pragma experimental ABIEncoderV2;

import "./Decoder.sol";
import "./Application.sol";
import "./Verifier.sol";

contract Bridge {
    using Decoder for bytes;

    // 0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470 is returned for accounts without code
    bytes32 NON_CONTRACT_ACCOUNT_HASH =
        0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470;

    mapping(bytes32 => bool) public eventHashes;
    mapping(address => bool) public applications;
    Verifier verifier;

    constructor(address _verfierAddr, address[] memory _apps) public {
        verifier = Verifier(_verfierAddr);
        for (uint256 i = 0; i < _apps.length; i++) {
            registerApp(_apps[i]);
        }
    }

    /**
     * @dev routes the message to the specified application ID after verifying the operator's signature
     * @param _appId address _appId
     * @param _message bytes _message contains information to be passed to the application
     */
    function submit(address _appId, bytes memory _message) public {
        require(
            verifier.verifyOperator(),
            "Tx origin must be the Bridge operator"
        );
        require(
            applications[_appId],
            "App ID not found. Only registered application are supported"
        );

        bytes32 hashed = keccak256(_message);
        require(!eventHashes[hashed], "Event has already been processed");
        eventHashes[hashed] = true;

        // Slice payload, dropping the 14-byte verification data
        bytes memory payload = _message.slice(0, _message.length - 15);
        Application app = Application(_appId);
        app.handle(payload);
    }

    /**
     * @dev registers a new application onto the bridge
     * @param _appID address _appID is the application's contract address to be registered
     */
    function registerApp(address _appID) internal {
        require(!applications[_appID], "Application is already registered");

        bytes32 codehash;
        assembly {
            codehash := extcodehash(_appID)
        }
        require(
            codehash != 0x0,
            "There's no account for this address on the network"
        );
        require(
            codehash != NON_CONTRACT_ACCOUNT_HASH,
            "Only contract accounts can be registered as applications"
        );

        Application app = Application(_appID);
        app.register(address(this));
        applications[_appID] = true;
    }
}
