// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

// Mock that always returns true (quorum met)
contract QuorumMockSuccess {
    fallback() external {
        // Return success (staticcall returns true)
        // No revert, just return
    }
}

// Mock that always returns false (quorum not met)
contract QuorumMockFail {
    fallback() external {
        // Revert to simulate quorum failure
        revert("Quorum not met");
    }
}
