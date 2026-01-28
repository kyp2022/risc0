// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.20;

// 引入 ERC721 标准 (使用 OpenZeppelin 风格的简化接口，实际部署建议使用完整库)
interface IERC721 {
    function ownerOf(uint256 tokenId) external view returns (address);
}

// 引入本地复制的 RISC Zero 合约
import {IRiscZeroVerifier} from "./IRiscZeroVerifier.sol";
import {RiscZeroGroth16Verifier} from "./RiscZeroGroth16Verifier.sol";
import {ControlID} from "./ControlID.sol";

contract GhostLinkSBT {
    // --- 状态变量 ---

    // Token 名称和符号
    string public name = "GhostLink GitHub Pass";
    string public symbol = "GHOST";

    // RISC Zero 验证器路由合约地址
    IRiscZeroVerifier public immutable verifier;

    // 你的 Guest 代码的 Image ID (部署时传入，或者后续通过 owner 更新)
    bytes32 public imageId;

    // 记录 Nullifier 是否已使用 (防止双花)
    // Key: Nullifier (GitHub ID 的哈希), Value: 是否已 Mint
    mapping(bytes32 => bool) public nullifiers;

    // 记录每个地址持有的 Token ID (简化版，每人只能持有一个)
    mapping(address => uint256) public balances;
    mapping(uint256 => address) public owners;

    // Token ID 计数器
    uint256 private _tokenIdCounter;

    // 事件
    event Minted(address indexed to, uint256 tokenId, bytes32 nullifier);

    // --- 构造函数 ---

    // 修改：移除了 _verifier 参数，默认自动部署新的 Verifier
    constructor(bytes32 _imageId) {
        // 自动部署一个新的 RiscZeroGroth16Verifier
        RiscZeroGroth16Verifier newVerifier = new RiscZeroGroth16Verifier(
            ControlID.CONTROL_ROOT,
            ControlID.BN254_CONTROL_ID
        );
        verifier = IRiscZeroVerifier(address(newVerifier));

        imageId = _imageId;
        _tokenIdCounter = 1;
    }

    // --- 核心功能 ---

    /**
     * @notice 验证 ZK 证明并铸造 SBT
     * @param seal RISC Zero 生成的 Groth16 证明
     * @param journal 公开输出 (包含 Nullifier)
     */
    function mint(bytes calldata seal, bytes calldata journal) external {
        // 1. 检查用户是否已经持有 SBT (可选限制)
        require(balances[msg.sender] == 0, "Already hold a pass");

        // 2. 验证 ZK 证明
        // 计算 journal 的 SHA-256 摘要
        bytes32 journalDigest = sha256(journal);

        // 调用 RISC Zero 验证器
        // 如果验证失败，这里会 Revert
        verifier.verify(seal, imageId, journalDigest);

        // 3. 解析 Journal 获取 Nullifier
        // 假设 Journal 的前 32 字节是 Nullifier (根据你的 Rust 代码逻辑调整)
        // 在 Rust 中我们输出了 hex string，但在上链时最好直接输出 bytes
        // 这里假设 journal 包含原始字节。
        // 如果 journal 是 "github_id_12345" 这样的字符串，我们需要对其进行哈希作为 key
        bytes32 nullifier = keccak256(journal);

        // 4. 防重放检查
        require(!nullifiers[nullifier], "Identity already used");

        // 5. 铸造 SBT
        uint256 tokenId = _tokenIdCounter;
        _tokenIdCounter++;

        balances[msg.sender] = 1;
        owners[tokenId] = msg.sender;
        nullifiers[nullifier] = true;

        emit Minted(msg.sender, tokenId, nullifier);
    }

    // --- SBT 限制 (禁止转账) ---

    function transferFrom(address, address, uint256) external pure {
        revert("SBT: transfer not allowed");
    }

    function safeTransferFrom(address, address, uint256) external pure {
        revert("SBT: transfer not allowed");
    }

    // --- 视图函数 ---

    function ownerOf(uint256 tokenId) external view returns (address) {
        address owner = owners[tokenId];
        require(owner != address(0), "Token does not exist");
        return owner;
    }
}
