// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.0;

/// @dev This mock contract simulates the behavior of the PodTxInfo contract
/// It returns fixed values for nonce and txHash for testing purposes
contract PodTxInfoMock {
    // Returns a fixed nonce and txHash
    fallback() external {
        // nonce = 42, txHash = bytes32(uint256(123))
        assembly {
            mstore(0x00, 42) // nonce at offset 0
            mstore(0x20, 123) // txHash at offset 32
            return(0x00, 0x40)
        }
    }
}

contract PodTxInfoWrongMock {
    // Returns wrong nonce and txHash
    fallback() external {
        assembly {
            mstore(0x00, 99) // wrong nonce
            mstore(0x20, 456) // wrong txHash
            return(0x00, 0x40)
        }
    }
}
