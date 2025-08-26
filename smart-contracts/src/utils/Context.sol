// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

/// @notice This library provides transaction related context
library Context {
    // ==============================================
    // Constants
    // ==============================================

    /// @dev This is the address obtained be performing this operation: address(uint160(uint256(keccak256("POD_TX_INFO"))));
    address constant POD_TX_INFO = 0x7687A3413739715807812b529f2d5f7Ef9057697;

    /// @notice This struct contains transaction related information
    /// @param nonce The transaction nonce
    /// @param txHash The transaction hash
    struct TxInfo {
        uint64 nonce;
        bytes32 txHash;
    }

    /// @notice Function gets information about the current transaction
    /// @return info The transaction information
    function getTxInfo() public view returns (TxInfo memory info) {
        /// @solidity memory-safe-assembly
        assembly {
            // Allocate memory for return data
            info := mload(0x40)
            mstore(0x40, add(info, 0x40))

            // Make staticcall and revert on failure in one line
            if iszero(staticcall(gas(), POD_TX_INFO, 0, 0, info, 0x40)) { revert(0, 0) }
        }
    }
}
