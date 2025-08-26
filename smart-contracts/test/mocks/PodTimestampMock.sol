// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.0;

// Mock that returns a fixed timestamp
contract TimeMockSuccess {
    fallback() external {
        // Return 1234567890123456 as uint256 (microseconds)
        assembly {
            mstore(0x00, 1234567890123456)
            return(0x00, 0x20)
        }
    }
}
