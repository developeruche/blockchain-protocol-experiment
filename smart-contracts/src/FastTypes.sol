// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.0;

import {Quorum} from "./Quorum.sol";

library FastTypes {
    // =====================================================
    // Structs/ Fast Types
    // =====================================================
    struct SharedCounter {
        mapping(bytes32 => uint256) _values;
    }

    struct OwnedCounter {
        mapping(bytes32 => mapping(address => uint256)) _values;
    }

    struct Balance {
        mapping(bytes32 => mapping(address => int256)) _values;
    }

    struct Set {
        mapping(bytes32 => uint256) _index;
        uint256 _length;
    }

    struct Uint256Set {
        Set _set;
        uint256 _maxValue;
    }

    struct AddressSet {
        Set _set;
    }

    // ==========================================
    // Fast Types Methods
    // ==========================================

    // ============ SharedCounter ===============
    function increment(SharedCounter storage c, bytes32 key, uint256 value) internal {
        c._values[key] += value;
    }

    function requireGte(SharedCounter storage c, bytes32 key, uint256 value, string memory errorMessage)
        internal
        view
    {
        Quorum.requireQuorum(c._values[key] >= value, errorMessage);
    }

    // ============ OwnedCounter ===============
    function get(OwnedCounter storage c, bytes32 key, address owner) internal view returns (uint256) {
        require(owner == tx.origin, "Cannot access OwnedCounter owned by another address");
        return c._values[key][owner];
    }

    function increment(OwnedCounter storage c, bytes32 key, address owner, uint256 value) internal {
        require(owner == tx.origin, "Cannot access OwnedCounter owned by another address");
        c._values[key][owner] += value;
    }

    function decrement(OwnedCounter storage c, bytes32 key, address owner, uint256 value) internal {
        require(owner == tx.origin, "Cannot access OwnedCounter owned by another address");
        require(c._values[key][owner] >= value, "Cannot decrement counter below 0");
        c._values[key][owner] -= value;
    }

    function set(OwnedCounter storage c, bytes32 key, address owner, uint256 value) internal {
        require(owner == tx.origin, "Cannot access OwnedCounter owned by another address");
        c._values[key][owner] = value;
    }

    // ============ Balance ===============
    function requireGte(Balance storage b, bytes32 key, address owner, uint256 value, string memory errorMessage)
        internal
        view
    {
        Quorum.requireQuorum(b._values[key][owner] >= int256(value), errorMessage);
    }

    function increment(Balance storage b, bytes32 key, address owner, uint256 value) internal {
        b._values[key][owner] += int256(value);
    }

    function decrement(Balance storage b, bytes32 key, address owner, uint256 value) internal {
        requireGte(b, key, owner, value, "Cannot decrement balance below 0");
        b._values[key][owner] -= int256(value);
    }

    // ============ Set ===============
    function add(Set storage s, bytes32 value) internal {
        if (s._index[value] == 0) {
            s._index[value] = s._length + 1;
            s._length++;
        }
    }

    function requireExists(Set storage s, bytes32 value, string memory errorMessage) internal view {
        Quorum.requireQuorum(s._index[value] > 0, errorMessage);
    }

    function requireLengthGte(Set storage s, uint256 value, string memory errorMessage) internal view {
        Quorum.requireQuorum(s._length >= value, errorMessage);
    }

    // ============ Uint256Set ===============

    function add(Uint256Set storage s, uint256 value) internal {
        if (value > s._maxValue) {
            s._maxValue = value;
        }

        add(s._set, bytes32(value));
    }

    function requireExists(Uint256Set storage s, uint256 value, string memory errorMessage) internal view {
        requireExists(s._set, bytes32(value), errorMessage);
    }

    function requireLengthGte(Uint256Set storage s, uint256 value, string memory errorMessage) internal view {
        requireLengthGte(s._set, value, errorMessage);
    }

    function requireMaxValueGte(Uint256Set storage s, uint256 value, string memory errorMessage) internal view {
        Quorum.requireQuorum(s._maxValue >= value, errorMessage);
    }

    // ============ AddressSet ===============

    function add(AddressSet storage s, address value) internal {
        add(s._set, bytes32(uint256(uint160(value))));
    }

    function requireExists(AddressSet storage s, address value, string memory errorMessage) internal view {
        requireExists(s._set, bytes32(uint256(uint160(value))), errorMessage);
    }

    function requireLengthGte(AddressSet storage s, uint256 value, string memory errorMessage) internal view {
        requireLengthGte(s._set, value, errorMessage);
    }
}
