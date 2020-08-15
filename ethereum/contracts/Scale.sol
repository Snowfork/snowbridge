// SPDX-License-Identifier: MIT
pragma solidity >=0.6.2;

contract Scale {

    event LogData(bytes _data);

    // decodeAddress
    function decodeAddress(bytes memory data)
        public
        pure
        returns (bytes memory)
    {
        // TODO: do something with data
        bytes memory decodedData = data;
        return decodedData;
    }

    function decodeBytes(bytes memory data)
        public
        returns (bytes memory)
    {
        // bytes memory decodedData = data;
        bytes8 action;
        assembly {action := mload(add(data, 1))}
        if(keccak256(abi.encodePacked(action)) == keccak256(abi.encodePacked("0x04"))) {
            emit LogData("1");
        }

        // bytes prefix = data[0];
        // if(prefix == bytes("0x04")) {
        //     emit LogData("check valid");
        // }

        return data;
    }

    function parse32BytesToAddress(bytes memory data) public {
        address parsed;
        assembly {parsed := mload(add(data, 32))}
    }

    function parse32BytesToBytes32(bytes memory data) public {
        bytes32 parsed;
        assembly {parsed := mload(add(data, 32))}
    }

    // 100 = 0x64
    function decodeUint64(bytes memory data) public {
        // uint256 amount;
        // uint256 nonce;
        // {val: []byte{0xa8}, output: int64(168)},
        // {val: []byte{0x15, 0x01}, output: int64(277)},

    }
}
