// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

contract Counter {
    uint256 public value;
    event Updated(uint256 value);

    function set(uint256 newValue) external {
        value = newValue;
        emit Updated(newValue);
    }
}
