// SPDX-License-Identifier: MIT
pragma solidity ^0.8.25;

import "forge-std/Test.sol";
import "../../src/Time.sol";
import "../../src/Quorum.sol";

/// @dev A wrapper contract to expose Time library functions for testing
/// Without wrapping this we would not be able it init a new frame, this is heavily depended upon
/// by the Foundry testing framework.
contract TimeWrapper {
    using Time for Time.Timestamp;

    function wrappedCurrentTime() external view returns (Time.Timestamp) {
        return Time.currentTime();
    }

    // ============ Constructor Functions ============

    function wrappedFromSeconds(uint64 seconds_) external pure returns (Time.Timestamp) {
        return Time.fromSeconds(seconds_);
    }

    function wrappedFromMillis(uint64 milliseconds) external pure returns (Time.Timestamp) {
        return Time.fromMillis(milliseconds);
    }

    function wrappedFromMicros(uint64 microseconds) external pure returns (Time.Timestamp) {
        return Time.fromMicros(microseconds);
    }

    // ============ Conversion Functions ============

    function wrappedToSeconds(Time.Timestamp timestamp) external pure returns (uint64) {
        return Time.toSeconds(timestamp);
    }

    // ============ Arithmetic Functions ============

    function wrappedAddSeconds(Time.Timestamp timestamp, uint64 seconds_) external pure returns (Time.Timestamp) {
        return Time.addSeconds(timestamp, seconds_);
    }

    function wrappedAddMillis(Time.Timestamp timestamp, uint64 milliseconds) external pure returns (Time.Timestamp) {
        return Time.addMillis(timestamp, milliseconds);
    }

    function wrappedAddMicros(Time.Timestamp timestamp, uint64 microseconds) external pure returns (Time.Timestamp) {
        return Time.addMicros(timestamp, microseconds);
    }

    function wrappedSubSeconds(Time.Timestamp timestamp, uint64 seconds_) external pure returns (Time.Timestamp) {
        return Time.subSeconds(timestamp, seconds_);
    }

    function wrappedSubMillis(Time.Timestamp timestamp, uint64 milliseconds) external pure returns (Time.Timestamp) {
        return Time.subMillis(timestamp, milliseconds);
    }

    function wrappedSubMicros(Time.Timestamp timestamp, uint64 microseconds) external pure returns (Time.Timestamp) {
        return Time.subMicros(timestamp, microseconds);
    }

    // ============ Comparison Functions ============

    function wrappedEq(Time.Timestamp a, Time.Timestamp b) external pure returns (bool) {
        return Time.eq(a, b);
    }

    function wrappedGt(Time.Timestamp a, Time.Timestamp b) external pure returns (bool) {
        return Time.gt(a, b);
    }

    function wrappedLt(Time.Timestamp a, Time.Timestamp b) external pure returns (bool) {
        return Time.lt(a, b);
    }

    function wrappedGte(Time.Timestamp a, Time.Timestamp b) external pure returns (bool) {
        return Time.gte(a, b);
    }

    function wrappedLte(Time.Timestamp a, Time.Timestamp b) external pure returns (bool) {
        return Time.lte(a, b);
    }

    // ============ Utility Functions ============

    function wrappedMin() external pure returns (Time.Timestamp) {
        return Time.min();
    }

    function wrappedMax() external pure returns (Time.Timestamp) {
        return Time.max();
    }

    function wrappedIsZero(Time.Timestamp timestamp) external pure returns (bool) {
        return Time.isZero(timestamp);
    }

    function wrappedBetween(Time.Timestamp timestamp, Time.Timestamp lower, Time.Timestamp upper)
        external
        pure
        returns (bool)
    {
        return Time.between(timestamp, lower, upper);
    }

    function wrappedMinOfTwo(Time.Timestamp a, Time.Timestamp b) external pure returns (Time.Timestamp) {
        return Time.min(a, b);
    }

    function wrappedMaxOfTwo(Time.Timestamp a, Time.Timestamp b) external pure returns (Time.Timestamp) {
        return Time.max(a, b);
    }

    // ============ Difference Functions ============

    function wrappedDiffMicros(Time.Timestamp a, Time.Timestamp b) external pure returns (uint64) {
        return Time.diffMicros(a, b);
    }

    function wrappedDiffMillis(Time.Timestamp a, Time.Timestamp b) external pure returns (uint64) {
        return Time.diffMillis(a, b);
    }

    function wrappedDiffSeconds(Time.Timestamp a, Time.Timestamp b) external pure returns (uint64) {
        return Time.diffSeconds(a, b);
    }

    // ============ Constants ============

    function getMicrosecondsPerSecond() external pure returns (uint64) {
        return Time.MICROSECONDS_PER_SECOND;
    }

    function getMicrosecondsPerMillisecond() external pure returns (uint64) {
        return Time.MICROSECONDS_PER_MILLISECOND;
    }

    function getMillisecondsPerSecond() external pure returns (uint64) {
        return Time.MILLISECONDS_PER_SECOND;
    }

    // ============ Method Chaining Examples ============

    function wrappedAddSecondsAndMillis(Time.Timestamp timestamp, uint64 seconds_, uint64 milliseconds)
        external
        pure
        returns (Time.Timestamp)
    {
        return timestamp.addSeconds(seconds_).addMillis(milliseconds);
    }

    function wrappedAddSecondsAndMicros(Time.Timestamp timestamp, uint64 seconds_, uint64 microseconds)
        external
        pure
        returns (Time.Timestamp)
    {
        return timestamp.addSeconds(seconds_).addMicros(microseconds);
    }

    function wrappedComplexChain(Time.Timestamp timestamp, uint64 seconds_, uint64 milliseconds, uint64 microseconds)
        external
        pure
        returns (Time.Timestamp)
    {
        return timestamp.addSeconds(seconds_).addMillis(milliseconds).addMicros(microseconds);
    }

    // ============ Helper Functions for Testing ============

    function createTimestampFromSeconds(uint64 seconds_) external pure returns (Time.Timestamp) {
        return Time.fromSeconds(seconds_);
    }

    function createTimestampFromMillis(uint64 milliseconds) external pure returns (Time.Timestamp) {
        return Time.fromMillis(milliseconds);
    }

    function createTimestampFromMicros(uint64 microseconds) external pure returns (Time.Timestamp) {
        return Time.fromMicros(microseconds);
    }

    function unwrapTimestamp(Time.Timestamp timestamp) external pure returns (uint64) {
        return Time.Timestamp.unwrap(timestamp);
    }

    function wrapTimestamp(uint64 value) external pure returns (Time.Timestamp) {
        return Time.Timestamp.wrap(value);
    }

    // ============ Require Time Functions ============

    function wrappedRequireTimeAfter(Time.Timestamp timestamp, string memory message) external view {
        Time.requireTimeAfter(timestamp, message);
    }

    function wrappedRequireTimeBefore(Time.Timestamp timestamp, string memory message) external view {
        Time.requireTimeBefore(timestamp, message);
    }
}
