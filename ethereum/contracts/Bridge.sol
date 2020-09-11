// SPDX-License-Identifier: MIT
pragma solidity >=0.6.2;

import "./Decoder.sol";
import "./Application.sol";

contract Bridge {
    using Decoder for bytes;

    uint64 public mostRecentBlock;
    mapping(uint64 => uint64) public mostRecentBlockEvent;
    mapping(address => bool) public applications;


    constructor(address[] memory apps) public {
        mostRecentBlock = 0;
        for(uint256 i = 0; i < apps.length; i++) {
            if(verifyApp(apps[i])) {
                applications[apps[i]] = true;
            }
        }
    }

    /**
     * @dev routes the message to the specified application ID after verifying the operator's signature
     * @param _message address _message expected type: Message { AppID [32]byte, Payload []byte }
     * @param _verification bytes _verification expected type: Verification { BlockNumber int64, EventID int64 }
     */
    function handle(bytes memory _message, bytes memory _verification)
        public
    {
        uint64 blockNumber = uint64(_verification.slice(0, 8).decodeUint256());
        require(blockNumber >= mostRecentBlock, "Blocks must be processed chronologically");

        uint64 eventID = uint64(_verification.slice(8, 16).decodeUint256());
        require(eventID > mostRecentBlockEvent[blockNumber], "Events must be processed chronologically");

        address appID = _message.sliceAddress(32);
        require(applications[appID], "App ID not found. Only registered application are supported");

        bytes memory payload = _message.slice(32, _message.length-1);
        Application app = Application(appID);
        app.handle(payload);

        mostRecentBlockEvent[blockNumber] = eventID;
        if(blockNumber > mostRecentBlock) {
            mostRecentBlock = blockNumber;
        }
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
        if(applications[_appID]){
            return false;
        }

        // 0x0 is the value returned for not-yet created accounts.
        // 0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470 is returned for accounts without code.
        bytes32 accountHash = 0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470;
        bytes32 codehash;
        assembly { codehash := extcodehash(_appID) }
        if(codehash != 0x0 && codehash != accountHash) {
            return false;
        }

        return true;
    }
}
