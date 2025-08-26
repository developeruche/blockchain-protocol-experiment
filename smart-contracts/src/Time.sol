// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import {Quorum} from "./Quorum.sol";

/// @title Time
/// @notice This library provides time-related utilities
/// @dev A library for working with timestamps in the POD network
library Time {
    using Time for Time.Timestamp;

    /// @dev Timestamp is a uint64 that represents the number of microseconds since the Unix epoch
    /// Usage: Timestamp ts = podTime.fromSeconds(1234567890);
    /// Valid range: [0, type(uint64).max]
    type Timestamp is uint64;

    // ==============================================
    // Constants
    // ==============================================

    /// @dev This is the address obtained be performing this operation: address(uint160(uint256(keccak256("POD_TIMESTAMP"))));
    address constant POD_TIMESTAMP = 0x423Bb123D9d5143e662606Fd343b6766d7BCf721;

    /// @dev Number of microseconds in a second
    uint64 constant MICROSECONDS_PER_SECOND = 1_000_000;

    /// @dev Number of microseconds in a millisecond
    uint64 constant MICROSECONDS_PER_MILLISECOND = 1_000;

    /// @dev Number of milliseconds in a second
    uint64 constant MILLISECONDS_PER_SECOND = 1_000;

    /// @dev The function obtains the timestamp from the POD system contract in microseconds
    /// @return ct The current timestamp in microseconds since the Unix epoch
    function currentTime() internal view returns (Timestamp ct) {
        uint256 rawTimestamp;

        /// @solidity memory-safe-assembly
        assembly {
            // Allocate memory for return data
            let ptr := mload(0x40)
            mstore(0x40, add(ptr, 0x20))

            // Make staticcall and revert on failure in one line
            if iszero(staticcall(gas(), POD_TIMESTAMP, 0, 0, ptr, 0x20)) { revert(0, 0) }

            // Load the returned timestamp
            rawTimestamp := mload(ptr)
        }

        ct = Timestamp.wrap(uint64(rawTimestamp));
    }

    /// @dev Obtain the the min value of Timestamp
    /// @return ts The minimum timestamp value
    function min() internal pure returns (Timestamp ts) {
        ts = Timestamp.wrap(0);
    }

    /// @dev Obtain the the max value of Timestamp
    /// @return ts The maximum timestamp value
    function max() internal pure returns (Timestamp ts) {
        ts = Timestamp.wrap(0xFFFFFFFFFFFFFFFF);
    }

    /// @dev Checks if a given timestamp if Zero
    /// @param timestamp the provided timestamp to check
    /// @return status true if timestamp is zero and false otherwise
    function isZero(Timestamp timestamp) internal pure returns (bool status) {
        return Timestamp.unwrap(timestamp) == 0;
    }

    /// @notice Creates a new timestamp from a raw timestamp provided in seasons
    /// @dev If the number of seconds is greater than the maximum value of uint64.max / MICROSECONDS_PER_SECOND, the function will revert.
    /// @param seconds_ The number of seconds the timestamp would be created from
    /// @return result This is the newly created timestamp
    function fromSeconds(uint64 seconds_) internal pure returns (Timestamp result) {
        /// @solidity memory-safe-assembly
        assembly {
            let mps := MICROSECONDS_PER_SECOND

            // Check for overflow: seconds_ > uint64.max / MICROSECONDS_PER_SECOND
            if gt(seconds_, div(0xffffffffffffffff, mps)) {
                // Revert with arithmetic overflow panic code
                mstore(0, 0x4e487b7100000000000000000000000000000000000000000000000000000000)
                mstore(0x04, 0x11) // Panic code for arithmetic overflow
                revert(0, 0x24)
            }

            // returning results
            result := mul(seconds_, mps)
        }
    }

    /// @notice Create a Timestamp from milliseconds
    /// @dev If the number of milliseconds is greater than the maximum value of uint64.max / MICROSECONDS_PER_MILLISECOND, the function will revert.
    /// @param milliseconds The number of milliseconds to create a Timestamp from
    /// @return result Calculated timestamp
    function fromMillis(uint64 milliseconds) internal pure returns (Timestamp result) {
        /// @solidity memory-safe-assembly
        assembly {
            let mpm := MICROSECONDS_PER_MILLISECOND

            // Check for overflow: seconds_ > uint64.max / MICROSECONDS_PER_SECOND
            if gt(milliseconds, div(0xffffffffffffffff, mpm)) {
                // Revert with arithmetic overflow panic code
                mstore(0, 0x4e487b7100000000000000000000000000000000000000000000000000000000)
                mstore(0x04, 0x11) // Panic code for arithmetic overflow
                revert(0, 0x24)
            }

            result := mul(milliseconds, mpm)
        }
    }

    /// @notice Create a Timestamp from microseconds
    /// @dev This is the default unit of timestamp
    /// @param microseconds The number of microseconds to create a Timestamp from
    /// @return Timestamp
    function fromMicros(uint64 microseconds) internal pure returns (Timestamp) {
        return Timestamp.wrap(microseconds);
    }

    /// @notice Convert Timestamp to seconds
    /// @dev If the Timestamp is not divisible by MICROSECONDS_PER_SECOND, the remainder is discarded.
    /// @param timestamp_ The Timestamp to convert
    /// @return secs The number of seconds
    function toSeconds(Timestamp timestamp_) internal pure returns (uint64 secs) {
        /// @solidity memory-safe-assembly
        assembly {
            secs := div(timestamp_, MICROSECONDS_PER_SECOND)
        }
    }

    /// @notice add seconds to a given Timestamp
    /// @param timestamp_ The original Timestamp
    /// @param seconds_ The number of seconds to add
    /// @return result The resulting Timestamp
    function addSeconds(Timestamp timestamp_, uint64 seconds_) internal pure returns (Timestamp result) {
        /// @solidity memory-safe-assembly
        assembly {
            let toAdd := mul(seconds_, MICROSECONDS_PER_SECOND)

            // Check for overflow
            if gt(toAdd, sub(0xffffffffffffffff, timestamp_)) {
                mstore(0, 0x4e487b7100000000000000000000000000000000000000000000000)
                mstore(0x04, 0x11) // Panic code for arithmetic underflow
                revert(0, 0x24)
            }

            result := add(timestamp_, toAdd)
        }
    }

    /// @notice add milliseconds to a given Timestamp
    /// @param timestamp_ The original Timestamp
    /// @param milliseconds The number of milliseconds to add
    /// @return result The resulting Timestamp
    function addMillis(Timestamp timestamp_, uint64 milliseconds) internal pure returns (Timestamp result) {
        /// @solidity memory-safe-assembly
        assembly {
            let toAdd := mul(milliseconds, MICROSECONDS_PER_MILLISECOND)

            // Check for overflow
            if gt(toAdd, sub(0xffffffffffffffff, timestamp_)) {
                mstore(0, 0x4e487b7100000000000000000000000000000000000000000000)
                mstore(0x04, 0x11) // Panic code for arithmetic underflow
                revert(0, 0x24)
            }

            result := add(timestamp_, toAdd)
        }
    }

    /// @notice add microseconds to a given Timestamp
    /// @param timestamp_ The original Timestamp
    /// @param microseconds The number of microseconds to add
    /// @return result The resulting Timestamp
    function addMicros(Timestamp timestamp_, uint64 microseconds) internal pure returns (Timestamp result) {
        /// @solidity memory-safe-assembly
        assembly {
            // Check for overflow
            if gt(microseconds, sub(0xffffffffffffffff, timestamp_)) {
                mstore(0, 0x4e487b7100000000000000000000000000000000000000000000)
                mstore(0x04, 0x11) // Panic code for arithmetic underflow
                revert(0, 0x24)
            }

            result := add(timestamp_, microseconds)
        }
    }

    /// @notice Subtract seconds from a given Timestamp
    /// @dev The function will revert if the subtraction results in a negative value.
    /// @param timestamp_ The original Timestamp
    /// @param seconds_ The number of seconds to subtract
    /// @return result The resulting Timestamp
    function subSeconds(Timestamp timestamp_, uint64 seconds_) internal pure returns (Timestamp result) {
        /// @solidity memory-safe-assembly
        assembly {
            let toSub := mul(seconds_, MICROSECONDS_PER_SECOND)
            // Revert if underflow
            if lt(timestamp_, toSub) {
                mstore(0, 0x4e487b7100000000000000000000000000000000000000000000000000000000)
                mstore(0x04, 0x11) // Panic code for arithmetic underflow
                revert(0, 0x24)
            }

            result := sub(timestamp_, toSub)
        }
    }

    /// @notice Subtract milliseconds from a given Timestamp
    /// @dev The function will revert if the subtraction results in a negative value.
    /// @param timestamp_ The original Timestamp
    /// @param milliseconds The number of milliseconds to subtract
    /// @return result The resulting Timestamp
    function subMillis(Timestamp timestamp_, uint64 milliseconds) internal pure returns (Timestamp result) {
        /// @solidity memory-safe-assembly
        assembly {
            let toSub := mul(milliseconds, MICROSECONDS_PER_MILLISECOND)
            // Revert if underflow
            if lt(timestamp_, toSub) {
                mstore(0, 0x4e487b7100000000000000000000000000000000000000000000000)
                mstore(0x04, 0x11) // Panic code for arithmetic underflow
                revert(0, 0x24)
            }

            result := sub(timestamp_, toSub)
        }
    }

    /// @notice Subtract microseconds from a given Timestamp
    /// @dev The function will revert if the subtraction results in a negative value.
    /// @param timestamp_ The original Timestamp
    /// @param microseconds The number of microseconds to subtract
    /// @return result The resulting Timestamp
    function subMicros(Timestamp timestamp_, uint64 microseconds) internal pure returns (Timestamp result) {
        /// @solidity memory-safe-assembly
        assembly {
            // Revert if underflow
            if lt(timestamp_, microseconds) {
                mstore(0, 0x4e487b7100000000000000000000000000000000000000000000000000000000)
                mstore(0x04, 0x11) // Panic code for arithmetic underflow
                revert(0, 0x24)
            }

            result := sub(timestamp_, microseconds)
        }
    }

    /// @notice Compares two timestamps for equality
    /// @param a The first timestamp
    /// @param b The second timestamp
    /// @return isEqual True if a == b, false otherwise
    function eq(Timestamp a, Timestamp b) internal pure returns (bool isEqual) {
        return Timestamp.unwrap(a) == Timestamp.unwrap(b);
    }

    /// @notice Compare two timestamps for which is greater
    /// @param a The first timestamp
    /// @param b The second timestamp
    /// @return isAGreater True if a > b, false otherwise
    function gt(Timestamp a, Timestamp b) internal pure returns (bool isAGreater) {
        return Timestamp.unwrap(a) > Timestamp.unwrap(b);
    }

    /// @notice Compare two timestamps for which is smaller
    /// @param a The first timestamp
    /// @param b The second timestamp
    /// @return isASmaller True if a < b, false otherwise
    function lt(Timestamp a, Timestamp b) internal pure returns (bool isASmaller) {
        return Timestamp.unwrap(a) < Timestamp.unwrap(b);
    }

    /// @notice Compare two timestamps for which is greater or equal
    /// @param a The first timestamp
    /// @param b The second timestamp
    /// @return isAGreaterOrEqual True if a >= b, false otherwise
    function gte(Timestamp a, Timestamp b) internal pure returns (bool isAGreaterOrEqual) {
        return Timestamp.unwrap(a) >= Timestamp.unwrap(b);
    }

    /// @notice Compare two timestamps for which is smaller or equal
    /// @param a The first timestamp
    /// @param b The second timestamp
    /// @return isASmallerOrEqual True if a <= b, false otherwise
    function lte(Timestamp a, Timestamp b) internal pure returns (bool isASmallerOrEqual) {
        return Timestamp.unwrap(a) <= Timestamp.unwrap(b);
    }

    /// @notice Ensures that a given timestamp is within a specified range
    /// @dev if the lower bound is greater than the upper bound the function will revert
    /// @param timestamp_ The timestamp to check
    /// @param lowerBound The inclusive lower bound of the range
    /// @param upperBound The inclusive upper bound of the range
    /// @return isInRange True if timestamp_ is within the range [lowerBound, upperBound], false otherwise
    function between(Timestamp timestamp_, Timestamp lowerBound, Timestamp upperBound)
        internal
        pure
        returns (bool isInRange)
    {
        /// @solidity memory-safe-assembly
        assembly {
            // Check that lowerBound <= upperBound
            if gt(lowerBound, upperBound) {
                // Revert with invalid argument panic code
                mstore(0, 0x4e487b7100000000000000000000000000000000000000000000000000000000)
                mstore(0x04, 0x32) // Panic code for invalid argument
                revert(0, 0x24)
            }

            // Check if timestamp_ is within the range [lowerBound, upperBound]
            // timestamp_ >= lowerBound AND timestamp_ <= upperBound
            isInRange :=
                and(
                    iszero(lt(timestamp_, lowerBound)), // timestamp_ >= lowerBound
                    iszero(gt(timestamp_, upperBound)) // timestamp_ <= upperBound
                )
        }
    }

    /// @notice Calculate the difference between two timestamps in microseconds
    /// @param a The first timestamp
    /// @param b The second timestamp
    /// @return diff The difference between the two timestamps (a, b)
    function diffMicros(Timestamp a, Timestamp b) internal pure returns (uint64 diff) {
        /// @solidity memory-safe-assembly
        assembly {
            if gt(a, b) { diff := sub(a, b) }
            if lt(a, b) { diff := sub(b, a) }
            if eq(a, b) { diff := 0 }
        }
    }

    /// @notice Calculate the difference between two timestamps in milliseconds
    /// @param a The first timestamp
    /// @param b The second timestamp
    /// @return diff The difference between the two timestamps (a, b) in milliseconds
    function diffMillis(Timestamp a, Timestamp b) internal pure returns (uint64 diff) {
        return diffMicros(a, b) / MICROSECONDS_PER_MILLISECOND;
    }

    /// @notice Calculate the difference between two timestamps in seconds
    /// @param a The first timestamp
    /// @param b The second timestamp
    /// @return diff The difference between the two timestamps (a, b) in seconds
    function diffSeconds(Timestamp a, Timestamp b) internal pure returns (uint64 diff) {
        return diffMicros(a, b) / MICROSECONDS_PER_SECOND;
    }

    /// @notice Get the minimum of two timestamps
    /// @param a The first timestamp
    /// @param b The second timestamp
    /// @return minTs The minimum of the two timestamps
    function min(Timestamp a, Timestamp b) internal pure returns (Timestamp minTs) {
        return lt(a, b) ? a : b;
    }

    /// @notice Get the maximum of two timestamps
    /// @param a The first timestamp
    /// @param b The second timestamp
    /// @return maxTs The maximum of the two timestamps
    function max(Timestamp a, Timestamp b) internal pure returns (Timestamp maxTs) {
        return gt(a, b) ? a : b;
    }

    /// @notice Requires that the current time is after a given timestamp
    /// @param _timestamp The timestamp to check against
    /// @param _message The message to revert with if the current time is not after the given timestamp
    function requireTimeAfter(Timestamp _timestamp, string memory _message) internal view {
        Quorum.requireQuorum(gt(currentTime(), _timestamp), _message);
    }

    /// @notice Requires that the current time is at least a given timestamp
    /// @param _timestamp The timestamp to check against
    /// @param _message The message to revert with if the current time is not at least
    function requireTimeAtLeast(Timestamp _timestamp, string memory _message) internal view {
        Quorum.requireQuorum(gte(currentTime(), _timestamp), _message);
    }

    /// @notice Requires that the current time is before a given timestamp
    /// @param _timestamp The timestamp to check against
    /// @param _message The message to revert with if the current time is not before the given timestamp
    function requireTimeBefore(Timestamp _timestamp, string memory _message) internal view {
        Quorum.requireQuorum(lt(currentTime(), _timestamp), _message);
    }

    /// @notice Requires that the current time is at most a given timestamp
    /// @param _timestamp The timestamp to check against
    /// @param _message The message to revert with if the current time is not at most
    function requireTimeAtMost(Timestamp _timestamp, string memory _message) internal view {
        Quorum.requireQuorum(lte(currentTime(), _timestamp), _message);
    }
}
