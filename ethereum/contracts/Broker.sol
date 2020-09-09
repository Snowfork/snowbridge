// SPDX-License-Identifier: MIT
pragma solidity >=0.6.2;

import "./Verifier.sol";
import "./Decoder.sol";
import "./Application.sol";

contract Broker {
    using Decoder for bytes;

    mapping(address => bool) public applications;
    Verifier public verifier;

    constructor(address verifierAddr) public {
        verifier = new Verifier(verifierAddr);
    }

    /**
     * @dev registers new applications
     * @param _appID address _appID is the contract address to be registered as a supported application
     */
    function registerApp(address _appID)
        public
    {
        require(!applications[_appID], "Application is already registered");

        // 0x0 is the value returned for not-yet created accounts.
        // 0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470 is returned for accounts without code.
        bytes32 accountHash = 0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470;
        bytes32 codehash;
        assembly { codehash := extcodehash(_appID) }
        require(codehash != 0x0 && codehash != accountHash, "Application must be a contract deployed to this network");

        // TODO: Consider checking that the application implements the abstract Application method 'submit' according
        //       to EIP-165 (incomplete implementation: https://github.com/ethereum/EIPs/blob/master/EIPS/eip-165.md)

        applications[_appID] = true;
    }

    /**
     * @dev routes the message to the specified application ID after verifying the operator's signature
     * @param _data address _data expected type: Message { AppID [32]byte, Payload []byte }
     * @param _signature address _signature expected type: Signature []byte
     */
    function submit(bytes memory _data, bytes memory _signature)
        public
    {
        require(_data.length > 32, "Data must contain an application ID and a message");

        address appID = _data.sliceAddress(32);
        require(applications[appID], "App ID not found. Has the application been registered?");

        bytes memory message = _data.slice(32, _data.length-1);
        require(verifier.verifyBytes(message, _signature), "Invalid operator signature");

        Application app = Application(appID);
        app.submit(_data);
    }
}
