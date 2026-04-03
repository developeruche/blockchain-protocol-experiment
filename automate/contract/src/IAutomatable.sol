// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

/// @title IAutomatable
/// @notice Any contract that wants automation must implement this.
interface IAutomatable {
    /// @notice Called by AutomateRegistry when a job fires.
    /// @param jobId  The bytes32 job identifier registered on the registry.
    /// @param data   Arbitrary calldata the job owner encoded at registration.
    /// @return success Must return true. Revert or return false to signal failure.
    function performAutomation(bytes32 jobId, bytes calldata data) external returns (bool success);
}
