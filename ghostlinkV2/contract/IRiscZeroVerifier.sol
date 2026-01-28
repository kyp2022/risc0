// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/**
 * @title IRiscZeroVerifier
 * @notice Interface for RISC Zero Verifier contracts
 * @dev This interface matches the RISC Zero Verifier Router contract
 *      Address on Sepolia: 0x925d8331ddc0a1F0d96E68CF073DFE1d92b69187
 *      
 *      IMPORTANT: The verify function does NOT return a bool.
 *      It reverts on verification failure, and succeeds silently on success.
 */
interface IRiscZeroVerifier {
    /**
     * @notice Verify a RISC Zero proof
     * @dev Reverts on verification failure. Returns nothing on success.
     * @param seal The proof seal (receipt) from RISC Zero (Groth16 format with selector)
     * @param imageId The Image ID of the guest program
     * @param journalHash The SHA-256 hash of the journal
     */
    function verify(
        bytes calldata seal,
        bytes32 imageId,
        bytes32 journalHash
    ) external view;
}
