// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/**
 * @title SimpleMockVerifier
 * @notice A mock verifier that accepts any proof as valid
 * @dev FOR DEVELOPMENT/TESTING ONLY - DO NOT USE IN PRODUCTION
 */
contract SimpleMockVerifier {
    /**
     * @notice Always returns true, accepting any proof
     * @param seal The proof seal (ignored)
     * @param imageId The guest program image ID (ignored)
     * @param journalHash The journal hash (ignored)
     * @return success Always returns true
     */
    function verify(
        bytes calldata seal,
        bytes32 imageId,
        bytes32 journalHash
    ) external pure returns (bool success) {
        // Suppress unused variable warnings
        seal;
        imageId;
        journalHash;
        
        return true;
    }
}
