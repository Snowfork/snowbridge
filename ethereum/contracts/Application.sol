 // SPDX-License-Identifier: MIT
pragma solidity >=0.6.2;

// Application contains methods that all applications must implement
abstract contract Application {
    function submit(bytes memory _data) public virtual;
}
