// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.0;

import {MerkleProof} from "@openzeppelin/contracts/utils/cryptography/MerkleProof.sol";

/// @title Merkle Tree Utilities
/// @notice Provides functions for hashing leaves and verifying Merkle proofs and multiproofs.
library MerkleTree {
    // ===============================================================
    // Structs
    // ===============================================================
    /// @dev Represents a Merkle proof with its path.
    /// @param path Array of sibling hashes from leaf to root
    struct Proof {
        bytes32[] path;
    }

    /// @dev Represents a Merkle multiproof with its path and flags.
    /// @param path Array of hashes used in multiproof verification
    /// @param flags Array of boolean flags for multiproof algorithm
    struct MultiProof {
        bytes32[] path;
        bool[] flags;
    }

    /// @notice Hashes a leaf node with a prefix using keccak256.
    /// @param prefix Optional prefix to prepend to the leaf
    /// @param leaf The leaf node to hash
    /// @return The resulting hash
    function hashLeaf(bytes memory prefix, bytes32 leaf) public pure returns (bytes32) {
        return keccak256(abi.encodePacked(prefix, leaf));
    }

    /// @notice Verifies a Merkle proof for a single leaf.
    /// @param root The Merkle root
    /// @param leaf The leaf node to verify
    /// @param proof The Merkle proof struct
    /// @return True if the proof is valid, false otherwise
    function verify(bytes32 root, bytes32 leaf, Proof memory proof) public pure returns (bool) {
        return MerkleProof.verify(proof.path, root, leaf);
    }

    /// @notice Verifies a Merkle multiproof for multiple leaves.
    /// @param root The Merkle root
    /// @param leaves Array of leaf nodes to verify
    /// @param proof The Merkle multiproof struct
    /// @return True if the multiproof is valid, false otherwise
    function verifyMulti(bytes32 root, bytes32[] memory leaves, MultiProof memory proof) public pure returns (bool) {
        return MerkleProof.multiProofVerify(proof.path, proof.flags, root, leaves);
    }
}
