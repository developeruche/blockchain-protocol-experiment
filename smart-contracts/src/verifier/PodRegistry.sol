// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";
import {IPodRegistry} from "../interfaces/IPodRegistry.sol";

/// @title PodRegistry Contract
/// @notice Manages a registry of validator addresses for a pod network, supporting addition, removal, and weight computation.
/// @dev Only the contract owner can add or remove validators. Fault tolerance and validator weight calculations are provided.
contract PodRegistry is IPodRegistry, Ownable {
    // ===============================================================
    // Constants
    // ===============================================================
    uint256 constant MAX_VALIDATOR_COUNT = 255;

    // ===============================================================
    // Storage
    // ===============================================================
    mapping(address => uint8) public validatorIndex;

    uint8 public validatorCount;
    uint8 public nextValidatorIndex;

    // ===============================================================
    // Events
    // ===============================================================
    /// @notice Emitted when a validator is added to the registry
    /// @param validator The address of the validator added
    event ValidatorAdded(address indexed validator);
    /// @notice Emitted when a validator is removed from the registry
    /// @param validator The address of the validator removed
    event ValidatorRemoved(address indexed validator);

    /// @notice Initializes the registry with a set of initial validators
    /// @param initialValidators Array of validator addresses to add at deployment
    constructor(address[] memory initialValidators) Ownable(msg.sender) {
        for (uint8 i = 0; i < initialValidators.length; i++) {
            addValidator(initialValidators[i]);
        }
    }

    /// @notice Adds a new validator to the registry
    /// @dev Only callable by the contract owner. Validator must not already exist and must not exceed max count.
    /// @param validator The address of the validator to add
    function addValidator(address validator) public onlyOwner {
        require(validator != address(0), "pod: validator is the zero address");
        require(validatorIndex[validator] == 0, "pod: validator already exists");
        require(nextValidatorIndex < MAX_VALIDATOR_COUNT, "pod: max validator count reached");
        validatorIndex[validator] = ++nextValidatorIndex;
        validatorCount++;
        emit ValidatorAdded(validator);
    }

    /// @notice Removes a validator from the registry
    /// @dev Only callable by the contract owner. Validator must exist in the registry.
    /// @param validator The address of the validator to remove
    function removeValidator(address validator) public onlyOwner {
        require(validatorIndex[validator] != 0, "pod: validator does not exist");
        delete validatorIndex[validator];
        validatorCount--;
        emit ValidatorRemoved(validator);
    }

    /// @notice Computes the weight of a subset of validators
    /// @dev Only counts unique, currently registered validators in the subset. Uses bitmask to avoid double-counting.
    /// @param subset Array of validator addresses to check
    /// @return weight Number of unique, valid validators in the subset
    function computeWeight(address[] memory subset) public view returns (uint256 weight) {
        uint256 counted = 0;
        for (uint8 i = 0; i < subset.length; i++) {
            uint8 index = validatorIndex[subset[i]];

            if (index == 0) {
                continue;
            }

            uint256 mask = 1 << (index - 1);
            if ((counted & mask) == 0) {
                counted |= mask;
                weight++;
            }
        }
    }

    /// @notice Returns the fault tolerance of the registry
    /// @dev Fault tolerance is defined as validatorCount / 3
    /// @return The number of faults tolerated
    function getFaultTolerance() external view returns (uint8) {
        return validatorCount / 3;
    }
}
