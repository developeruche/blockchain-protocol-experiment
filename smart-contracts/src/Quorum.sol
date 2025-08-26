// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.0;

/// @notice This library aids with the interaction with the Quorum system-contract/pre-compile
library Quorum {
    // ==============================================
    // Constants
    // ==============================================

    /// @dev This is the address obtained be performing this operation: address(uint160(uint256(keccak256("POD_REQUIRE_QUORUM"))));
    address constant POD_REQUIRE_QUORUM = 0x6AD9145E866c7A7DcCc6c277Ea86abBD268FBAc9;

    /// @notice Function requires that the current transaction meets the quorum requirements
    /// @param input The input boolean to pass to the quorum check
    /// @param message The error message to use if the quorum check fails
    function requireQuorum(bool input, string memory message) public view {
        /// @solidity memory-safe-assembly
        assembly {
            // Store input boolean in memory
            mstore(0x00, input)

            // Make staticcall and revert on failure in one line
            if iszero(staticcall(gas(), POD_REQUIRE_QUORUM, 0, 0x20, 0, 0)) {
                // Prepare the revert message
                let msgPtr := add(message, 0x20)
                let msgLen := mload(message)

                // Revert with the raw message data: TODO: it would be nice to make this a costom error
                revert(msgPtr, msgLen)
            }
        }
    }
}
