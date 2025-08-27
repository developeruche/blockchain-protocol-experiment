// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

/// @title IPodRegistry
/// @notice Interface for the PodRegistry contract
/// @dev Provides methods for validator subset weight and fault tolerance
interface IPodRegistry {
    /// @notice Computes the weight of a subset of validators
    /// @param subset Array of validator addresses to check
    /// @return weight Number of unique, valid validators in the subset
    function computeWeight(address[] memory subset) external view returns (uint256 weight);

    /// @notice Returns the fault tolerance of the registry
    /// @return The number of faults tolerated
    function getFaultTolerance() external view returns (uint8);
}
