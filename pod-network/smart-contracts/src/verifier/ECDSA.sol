// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.0;

/// @title ECDSA Signature Utilities
/// @notice Provides functions for serializing, deserializing, aggregating, and verifying ECDSA signatures.
library ECDSA {
    /// @dev Represents an ECDSA signature with its components.
    /// @param v Recovery id
    /// @param r First 32 bytes of the signature
    /// @param s Second 32 bytes of the signature
    struct Signature {
        uint8 v;
        bytes32 r;
        bytes32 s;
    }

    /// @notice Serializes signature components into a 65-byte array.
    /// @param v Recovery id
    /// @param r First 32 bytes of the signature
    /// @param s Second 32 bytes of the signature
    /// @return signature Serialized signature as bytes
    function _serialize_signature(uint8 v, bytes32 r, bytes32 s) internal pure returns (bytes memory signature) {
        signature = new bytes(65);
        assembly {
            mstore(add(signature, 32), r)
            mstore(add(signature, 64), s)
            mstore8(add(signature, 96), v)
        }
    }

    /// @notice Serializes a Signature struct into a 65-byte array.
    /// @param signature The Signature struct to serialize
    /// @return Serialized signature as bytes
    function serialize_signature(Signature memory signature) internal pure returns (bytes memory) {
        return _serialize_signature(signature.v, signature.r, signature.s);
    }

    /// @notice Serializes an array of Signature structs into an array of 65-byte arrays.
    /// @param signatures Array of Signature structs
    /// @return serialized Array of serialized signatures
    function serialize_signatures(Signature[] memory signatures) internal pure returns (bytes[] memory serialized) {
        serialized = new bytes[](signatures.length);
        for (uint256 i = 0; i < signatures.length; i++) {
            serialized[i] = _serialize_signature(signatures[i].v, signatures[i].r, signatures[i].s);
        }
    }

    /// @notice Deserializes a 65-byte signature into its components.
    /// @param signature Serialized signature as bytes
    /// @return v Recovery id
    /// @return r First 32 bytes of the signature
    /// @return s Second 32 bytes of the signature
    function _deserialize_signature(bytes memory signature) internal pure returns (uint8 v, bytes32 r, bytes32 s) {
        require(signature.length == 65, "invalid signature length");
        assembly {
            let ptr := add(signature, 32)
            r := mload(ptr)
            s := mload(add(ptr, 32))
            v := byte(0, mload(add(ptr, 64)))
        }
    }

    /// @notice Deserializes a 65-byte signature into a Signature struct.
    /// @param signature Serialized signature as bytes
    /// @return Deserialized Signature struct
    function deserialize_signature(bytes memory signature) internal pure returns (Signature memory) {
        (uint8 v, bytes32 r, bytes32 s) = _deserialize_signature(signature);
        return Signature(v, r, s);
    }

    /// @notice Concatenates multiple encoded ECDSA signatures into a single bytes array.
    /// @dev Each signature should be 65 bytes long.
    /// @param signatures Array of encoded signatures
    /// @return aggregate Concatenated signatures as bytes
    function aggregate_signatures(bytes[] memory signatures) internal pure returns (bytes memory aggregate) {
        uint256 signatureCount = signatures.length;
        aggregate = new bytes(signatureCount * 65);
        assembly {
            let signaturesPtr := add(signatures, 32)
            let aggregatePtr := add(aggregate, 32)

            for { let i := 0 } lt(i, signatureCount) { i := add(i, 1) } {
                let signature := mload(add(signaturesPtr, mul(i, 32)))

                if iszero(eq(mload(signature), 65)) {
                    mstore(0, 32) // offset for error
                    mstore(32, 23) // length of error string
                    mstore(64, "invalid signature length") // error string
                    revert(0, 64) // revert with error reason
                }

                mstore(aggregatePtr, mload(add(signature, 32)))
                mstore(add(aggregatePtr, 32), mload(add(signature, 64)))
                mstore8(add(aggregatePtr, 64), byte(0, mload(add(signature, 96))))

                aggregatePtr := add(aggregatePtr, 65)
            }
        }
    }

    /// @notice Splits an aggregate signature into its individual encoded ECDSA signatures.
    /// @param aggregate Concatenated signatures as bytes
    /// @return signatures Array of encoded signatures
    function disaggregate_signatures(bytes memory aggregate) internal pure returns (bytes[] memory signatures) {
        require(aggregate.length % 65 == 0, "invalid aggregate length");

        uint256 signatureCount = aggregate.length / 65;
        signatures = new bytes[](signatureCount);

        assembly {
            let aggregatePtr := add(aggregate, 32)

            for { let i := 0 } lt(i, signatureCount) { i := add(i, 1) } {
                let signature := mload(64)
                mstore(signature, 65)

                mstore(add(signature, 32), mload(aggregatePtr))
                mstore(add(signature, 64), mload(add(aggregatePtr, 32)))
                mstore8(add(signature, 96), byte(0, mload(add(aggregatePtr, 64))))

                mstore(add(signatures, mul(32, add(i, 1))), signature)
                aggregatePtr := add(aggregatePtr, 65)
                mstore(64, add(signature, 97))
            }
        }
    }

    /// @notice Recovers the signer address from a digest and signature.
    /// @param digest The message hash that was signed
    /// @param signature The Signature struct
    /// @return Address of the signer
    function recoverSigner(bytes32 digest, Signature memory signature) internal pure returns (address) {
        return ecrecover(digest, signature.v, signature.r, signature.s);
    }

    /// @notice Recovers signer addresses from a digest and an aggregate signature.
    /// @param digest The message hash that was signed
    /// @param aggregateSignature Concatenated signatures as bytes
    /// @return Array of signer addresses
    function recoverSigners(bytes32 digest, bytes memory aggregateSignature) internal pure returns (address[] memory) {
        bytes[] memory signatures = disaggregate_signatures(aggregateSignature);
        address[] memory signers = new address[](signatures.length);
        for (uint256 i = 0; i < signatures.length; i++) {
            signers[i] = recoverSigner(digest, deserialize_signature(signatures[i]));
        }

        return signers;
    }

    /// @notice Verifies that a signature was produced by a given signer for a digest.
    /// @param signer Expected signer address
    /// @param digest The message hash that was signed
    /// @param signature The Signature struct
    /// @return True if valid, false otherwise
    function verify(address signer, bytes32 digest, Signature memory signature) internal pure returns (bool) {
        return recoverSigner(digest, signature) == signer;
    }

    /// @notice Verifies that an aggregate signature was produced by a set of signers for a digest.
    /// @param signers Array of expected signer addresses
    /// @param digest The message hash that was signed
    /// @param aggregateSignature Concatenated signatures as bytes
    /// @return True if all signatures are valid and match the signers
    function verify(address[] memory signers, bytes32 digest, bytes memory aggregateSignature)
        internal
        pure
        returns (bool)
    {
        address[] memory recoveredSigners = recoverSigners(digest, aggregateSignature);
        if (recoveredSigners.length != signers.length) {
            return false;
        }

        for (uint256 i = 0; i < signers.length; i++) {
            if (recoveredSigners[i] != signers[i]) {
                return false;
            }
        }

        return true;
    }
}
