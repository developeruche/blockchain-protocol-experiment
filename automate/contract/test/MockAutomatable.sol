// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {IAutomatable} from "../src/IAutomatable.sol";

/// @notice Minimal IAutomatable for testing. Tracks calls and can be toggled to fail.
contract MockAutomatable is IAutomatable {
    address public immutable REGISTRY;

    uint256 public callCount;
    bytes32 public lastJobId;
    bytes public lastData;
    bool public shouldRevert;
    bool public shouldReturnFalse;

    event AutomationPerformed(bytes32 indexed jobId, bytes data, uint256 callCount);

    error OnlyRegistry();
    error ForcedFailure();

    constructor(address _registry) {
        REGISTRY = _registry;
    }

    function performAutomation(bytes32 jobId, bytes calldata data) external override returns (bool) {
        if (msg.sender != REGISTRY) revert OnlyRegistry();
        if (shouldRevert) revert ForcedFailure();
        if (shouldReturnFalse) return false;

        callCount++;
        lastJobId = jobId;
        lastData = data;

        emit AutomationPerformed(jobId, data, callCount);
        return true;
    }

    // Test helpers
    function setRevert(bool v) external {
        shouldRevert = v;
    }

    function setReturnFalse(bool v) external {
        shouldReturnFalse = v;
    }
}
