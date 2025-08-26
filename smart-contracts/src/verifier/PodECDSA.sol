// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.0;

import {ECDSA} from "./ECDSA.sol";
import {MerkleTree} from "./MerkleTree.sol";
import {IPodRegistry} from "./PodRegistry.sol";
import {Strings} from "@openzeppelin/contracts/utils/Strings.sol";

/// @title PodECDSA Library
/// @notice Provides utilities for verifying certified logs and receipts using ECDSA and Merkle proofs in a pod network.
library PodECDSA {
    /// @dev Pod configuration containing quorum and registry reference.
    /// @param quorum Minimum required weight for validation
    /// @param registry Pod registry contract
    struct PodConfig {
        uint256 quorum;
        IPodRegistry registry;
    }

    /// @dev Represents an event log with address, topics, and data.
    /// @param addr Address that emitted the log
    /// @param topics Array of indexed topics
    /// @param data Log data
    struct Log {
        address addr;
        bytes32[] topics;
        bytes data;
    }

    /// @dev Receipt certified by validators, containing root and aggregate signature.
    /// @param receiptRoot Merkle root of the receipt
    /// @param aggregateSignature Aggregate ECDSA signature of validators
    struct CertifiedReceipt {
        bytes32 receiptRoot;
        bytes aggregateSignature;
    }

    /// @dev Certificate for a leaf, including its proof and certified receipt.
    /// @param certifiedReceipt Certified receipt struct
    /// @param leaf Leaf node being certified
    /// @param proof Merkle proof for the leaf
    struct Certificate {
        CertifiedReceipt certifiedReceipt;
        bytes32 leaf;
        MerkleTree.Proof proof;
    }

    /// @dev Certified log with its index and certificate.
    /// @param log Log struct
    /// @param logIndex Index of the log in the batch
    /// @param certificate Certificate for the log
    struct CertifiedLog {
        Log log;
        uint256 logIndex;
        Certificate certificate;
    }

    /// @dev Certificate for multiple leaves, including multiproof and certified receipt.
    /// @param certifiedReceipt Certified receipt struct
    /// @param leaves Array of leaf nodes
    /// @param proof Merkle multiproof for the leaves
    struct MultiCertificate {
        CertifiedReceipt certifiedReceipt;
        bytes32[] leaves;
        MerkleTree.MultiProof proof;
    }

    /// @notice Hashes a log struct using keccak256.
    /// @param log Log struct to hash
    /// @return The hash of the log
    function hashLog(Log calldata log) public pure returns (bytes32) {
        return keccak256(abi.encode(log));
    }

    /// @notice Verifies a certified receipt against the pod configuration.
    /// @param podConfig Pod configuration struct
    /// @param certifiedReceipt Certified receipt to verify
    /// @return True if the receipt is valid and meets quorum
    function verifyCertifiedReceipt(PodConfig calldata podConfig, CertifiedReceipt calldata certifiedReceipt)
        public
        view
        returns (bool)
    {
        address[] memory validators =
            ECDSA.recoverSigners(certifiedReceipt.receiptRoot, certifiedReceipt.aggregateSignature);
        return podConfig.registry.computeWeight(validators) >= podConfig.quorum;
    }

    /// @notice Verifies a certificate for a leaf against the pod configuration.
    /// @param podConfig Pod configuration struct
    /// @param certificate Certificate to verify
    /// @return True if the certificate and its proof are valid
    function verifyCertificate(PodConfig calldata podConfig, Certificate calldata certificate)
        public
        view
        returns (bool)
    {
        return verifyCertifiedReceipt(podConfig, certificate.certifiedReceipt)
            && MerkleTree.verify(certificate.certifiedReceipt.receiptRoot, certificate.leaf, certificate.proof);
    }

    /// @notice Verifies a multi-certificate for multiple leaves against the pod configuration.
    /// @param podConfig Pod configuration struct
    /// @param certificate MultiCertificate to verify
    /// @return True if the multi-certificate and its multiproof are valid
    function verifyMultiCertificate(PodConfig calldata podConfig, MultiCertificate calldata certificate)
        public
        view
        returns (bool)
    {
        return verifyCertifiedReceipt(podConfig, certificate.certifiedReceipt)
            && MerkleTree.verifyMulti(certificate.certifiedReceipt.receiptRoot, certificate.leaves, certificate.proof);
    }

    /// @notice Verifies a certified log against the pod configuration and its certificate.
    /// @param podConfig Pod configuration struct
    /// @param certifiedLog CertifiedLog to verify
    /// @return True if the certified log and its certificate are valid
    function verifyCertifiedLog(PodConfig calldata podConfig, CertifiedLog calldata certifiedLog)
        public
        view
        returns (bool)
    {
        bytes32 logHash = hashLog(certifiedLog.log);

        bytes32 leaf = MerkleTree.hashLeaf(
            bytes(string.concat("log_hashes[", Strings.toString(certifiedLog.logIndex), "]")), logHash
        );

        require(leaf == certifiedLog.certificate.leaf, "Invalid certificate");

        return verifyCertificate(podConfig, certifiedLog.certificate);
    }
}