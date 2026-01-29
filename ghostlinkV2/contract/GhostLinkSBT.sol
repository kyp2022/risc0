// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/token/ERC721/ERC721.sol";
import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/utils/Strings.sol";
import "./IRiscZeroVerifier.sol";

/**
 * @title GhostLinkSBT
 * @notice Soul Bound Token (SBT) contract for GhostLink credentials
 * @dev Non-transferable ERC721 tokens representing verified credentials
 */
contract GhostLinkSBT is ERC721, Ownable {
    using Strings for uint256;

    // ============ Types ============
    
    enum CredentialType {
        GITHUB,     // 0
        ALIPAY,     // 1
        TWITTER,    // 2
        WALLET      // 3
    }

    struct Credential {
        CredentialType credType;   // 凭证类型
        uint256 mintedAt;          // 铸造时间戳
        bytes32 nullifier;         // 唯一标识符（防重复）
    }

    // ============ State Variables ============
    
    /// @notice RISC Zero Verifier contract address
    IRiscZeroVerifier public verifier;
    
    /// @notice Guest program Image ID
    bytes32 public imageId;
    
    /// @notice Base URI for token metadata
    string private _baseTokenURI;
    
    /// @notice Track used nullifiers to prevent double minting
    mapping(bytes32 => bool) public nullifiers;
    
    /// @notice Token ID to credential mapping
    mapping(uint256 => Credential) public credentials;
    
    /// @notice User address to token IDs mapping
    mapping(address => uint256[]) public userTokens;
    
    /// @notice Total supply counter
    uint256 private _tokenIdCounter;

    // ============ Events ============
    
    event Minted(
        address indexed recipient,
        uint256 indexed tokenId,
        bytes32 indexed nullifier,
        CredentialType credType
    );
    
    event ImageIdUpdated(bytes32 oldId, bytes32 newId);
    event VerifierUpdated(address oldVerifier, address newVerifier);
    event BaseURIUpdated(string oldURI, string newURI);

    // ============ Constructor ============
    
    /**
     * @param _verifier RISC Zero Verifier Router address
     * @param _imageId Guest program Image ID
     * @param _baseURI Base URI for token metadata
     * @param _name Token name
     * @param _symbol Token symbol
     */
    constructor(
        address _verifier,
        bytes32 _imageId,
        string memory _baseURI,
        string memory _name,
        string memory _symbol
    ) ERC721(_name, _symbol) Ownable(msg.sender) {
        require(_verifier != address(0), "Invalid verifier address");
        require(_imageId != bytes32(0), "Invalid image ID");
        
        verifier = IRiscZeroVerifier(_verifier);
        imageId = _imageId;
        _baseTokenURI = _baseURI;
        _tokenIdCounter = 1; // Start from token ID 1
    }

    // ============ Core Functions ============
    
    /**
     * @notice Mint a new credential SBT
     * @param seal ZK proof seal (receipt) from RISC Zero
     * @param nullifier Unique identifier to prevent double minting
     * @param credType Credential type (0: GitHub, 1: Alipay, 2: Twitter, 3: Wallet)
     * @return tokenId The minted token ID
     */
    function mint(
        bytes calldata seal,
        bytes32 nullifier,
        CredentialType credType
    ) external returns (uint256 tokenId) {
        // 1. Check nullifier not used
        require(!nullifiers[nullifier], "Already minted");
        
        // 2. Construct journal hash
        // Journal format: abi.encodePacked(msg.sender, nullifier, uint8(credType))
        // IMPORTANT: RISC Zero on-chain verifier requires SHA-256 for the journal hash
        // This must match the journal format in guest/src/main.rs (53 bytes)
        bytes32 journalHash = sha256(
            abi.encodePacked(msg.sender, nullifier, uint8(credType))
        );

        // 3. Verify ZK proof
        // NOTE: verifier.verify() reverts on failure, returns nothing on success
        // The seal must be in Groth16 format: [4-byte selector][Groth16 proof data]
        verifier.verify(seal, imageId, journalHash);

        // 4. Mark nullifier as used
        nullifiers[nullifier] = true;

        // 5. Mint token
        tokenId = _tokenIdCounter++;
        _mint(msg.sender, tokenId);

        // 6. Record credential info
        credentials[tokenId] = Credential({
            credType: credType,
            mintedAt: block.timestamp,
            nullifier: nullifier
        });

        // 7. Update user tokens mapping
        userTokens[msg.sender].push(tokenId);

        // 8. Emit event
        emit Minted(msg.sender, tokenId, nullifier, credType);

        return tokenId;
    }

    // ============ Query Functions ============

    /**
     * @notice Get all credentials for a user
     * @param user User address
     * @return creds Array of credentials
     */
    function getCredentials(address user)
        external
        view
        returns (Credential[] memory creds)
    {
        uint256[] memory tokenIds = userTokens[user];
        creds = new Credential[](tokenIds.length);

        for (uint256 i = 0; i < tokenIds.length; i++) {
            creds[i] = credentials[tokenIds[i]];
        }

        return creds;
    }

    /**
     * @notice Check if user has a specific credential type
     * @param user User address
     * @param credType Credential type to check
     * @return hasCredential True if user has the credential type
     */
    function hasCredentialType(address user, CredentialType credType)
        external
        view
        returns (bool hasCredential)
    {
        uint256[] memory tokenIds = userTokens[user];

        for (uint256 i = 0; i < tokenIds.length; i++) {
            if (credentials[tokenIds[i]].credType == credType) {
                return true;
            }
        }

        return false;
    }

    /**
     * @notice Get credential info for a specific token
     * @param tokenId Token ID
     * @return cred Credential struct
     */
    function getCredential(uint256 tokenId)
        external
        view
        returns (Credential memory cred)
    {
        require(_ownerOf(tokenId) != address(0), "Token does not exist");
        return credentials[tokenId];
    }

    /**
     * @notice Debug helper to calculate the expected journal hash
     * @param user The address of the recipient
     * @param nullifier The nullifier from the ZK proof
     * @param credType The type of credential
     * @return The calculated SHA-256 hash
     */
    function calculateJournalHash(
        address user,
        bytes32 nullifier,
        CredentialType credType
    ) public pure returns (bytes32) {
        return sha256(abi.encodePacked(user, nullifier, uint8(credType)));
    }

    /**
     * @notice Get token IDs owned by a user
     * @param user User address
     * @return tokenIds Array of token IDs
     */
    function getUserTokenIds(address user) 
        external 
        view 
        returns (uint256[] memory tokenIds) 
    {
        return userTokens[user];
    }

    // ============ Token URI ============
    
    /**
     * @notice Get token URI for metadata
     * @param tokenId Token ID
     * @return Token URI string
     */
    function tokenURI(uint256 tokenId) 
        public 
        view 
        override 
        returns (string memory) 
    {
        require(_ownerOf(tokenId) != address(0), "Token does not exist");
        
        Credential memory cred = credentials[tokenId];
        string memory baseURI = _baseTokenURI;
        
        // Return different metadata based on credential type
        if (cred.credType == CredentialType.GITHUB) {
            return string(abi.encodePacked(baseURI, "github.json"));
        } else if (cred.credType == CredentialType.ALIPAY) {
            return string(abi.encodePacked(baseURI, "alipay.json"));
        } else if (cred.credType == CredentialType.TWITTER) {
            return string(abi.encodePacked(baseURI, "twitter.json"));
        } else {
            return string(abi.encodePacked(baseURI, "wallet.json"));
        }
    }

    // ============ Admin Functions ============
    
    /**
     * @notice Update Image ID (for guest program upgrades)
     * @param newImageId New Image ID
     */
    function setImageId(bytes32 newImageId) external onlyOwner {
        require(newImageId != bytes32(0), "Invalid image ID");
        bytes32 oldId = imageId;
        imageId = newImageId;
        emit ImageIdUpdated(oldId, newImageId);
    }
    
    /**
     * @notice Update Verifier address
     * @param newVerifier New verifier address
     */
    function setVerifier(address newVerifier) external onlyOwner {
        require(newVerifier != address(0), "Invalid verifier address");
        address oldVerifier = address(verifier);
        verifier = IRiscZeroVerifier(newVerifier);
        emit VerifierUpdated(oldVerifier, newVerifier);
    }
    
    /**
     * @notice Update base token URI
     * @param newBaseURI New base URI
     */
    function setBaseTokenURI(string calldata newBaseURI) external onlyOwner {
        string memory oldURI = _baseTokenURI;
        _baseTokenURI = newBaseURI;
        emit BaseURIUpdated(oldURI, newBaseURI);
    }

    // ============ SBT Override (Non-transferable) ============
    
    /**
     * @notice Override to prevent transfers (SBT behavior)
     * @dev Only allow minting (from = address(0)) and burning (to = address(0))
     */
    function _update(
        address to,
        uint256 tokenId,
        address auth
    ) internal override returns (address) {
        address from = _ownerOf(tokenId);
        
        // Allow minting (from == address(0)) and burning (to == address(0))
        // Prevent transfers (both from and to are non-zero)
        require(
            from == address(0) || to == address(0),
            "SBT: non-transferable"
        );
        
        return super._update(to, tokenId, auth);
    }
    
    /**
     * @notice Override to prevent approvals
     */
    function approve(address, uint256) public pure override {
        revert("SBT: approvals not allowed");
    }
    
    /**
     * @notice Override to prevent operator approvals
     */
    function setApprovalForAll(address, bool) public pure override {
        revert("SBT: operator approvals not allowed");
    }

    // ============ View Functions ============
    
    /**
     * @notice Get total supply
     * @return Total number of minted tokens
     */
    function totalSupply() external view returns (uint256) {
        return _tokenIdCounter - 1;
    }
}
