// SPDX-License-Identifier: MIT
pragma solidity >=0.6.2;

import "./Decoder.sol";
import "./Application.sol";

contract Bridge {
    using Decoder for bytes;

    // 0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470 is returned for accounts without code
    bytes32 NON_CONTRACT_ACCOUNT_HASH = 0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470;

    mapping(bytes32 => bool) public eventHashes;
    mapping(address => bool) public applications;

    constructor(address[] memory apps) public {
        for(uint256 i = 0; i < apps.length; i++) {
            if(verifyApp(apps[i])) {
                applications[apps[i]] = true;
            }
        }
    }

    /**
     * @dev routes the message to the specified application ID after verifying the operator's signature
     * @param _message address _message expected type: Message { AppID [32]byte, Payload []byte }
     * @param _hash bytes _hash is a unique hash of the event (app_id, payload, blockNum, eventId)
     */
    function submit(bytes memory _message, bytes32 _hash)
        public
    {
        address appID = _message.sliceAddress(32);
        require(applications[appID], "App ID not found. Only registered application are supported");

        require(!eventHashes[_hash], "Event has already been processed");
        eventHashes[_hash] = true;

        bytes memory payload = _message.slice(32, _message.length-1);
        Application app = Application(appID);
        app.handle(payload);
    }

    /**
     * @dev verifies new applications
     * @param _appID address _appID is the application's contract address to be verified
     */
    function verifyApp(address _appID)
        internal
        view
        returns(bool)
    {
        require(applications[_appID], "Application is already registered");

        bytes32 codehash;
        assembly { codehash := extcodehash(_appID) }
        require(codehash != 0x0, "There's no account for this address on the network");
        require(codehash != NON_CONTRACT_ACCOUNT_HASH, "Only contract accounts can be registered as applications");

        return true;
    }
}
