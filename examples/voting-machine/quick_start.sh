#!/bin/bash

# 投票机示例：快速开始指南
# 
# 这个脚本帮助你快速运行和理解投票机示例
# 
# 使用方法：
#   bash quick_start.sh          # 运行完整测试
#   bash quick_start.sh check    # 仅检查代码
#   bash quick_start.sh doc      # 打开文档

set -e

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_header() {
    echo -e "${BLUE}════════════════════════════════════════${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}════════════════════════════════════════${NC}"
}

print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠ $1${NC}"
}

print_info() {
    echo -e "${BLUE}ℹ $1${NC}"
}

# 子命令
CMD=${1:-test}

case "$CMD" in
    test)
        print_header "运行投票机示例测试"
        print_info "这将执行完整的投票协议测试并显示日志"
        echo ""
        
        echo "编译和运行测试（第一次运行可能需要几分钟）..."
        RUST_LOG=info cargo test --release -- --nocapture
        
        print_success "测试通过！"
        echo ""
        print_info "关键输出："
        echo "  - [INFO] init: 初始化投票机"
        echo "  - [INFO] submit: 提交选票"
        echo "  - [INFO] freeze: 冻结投票站"
        echo "  - Final vote count: 最终票数"
        ;;
    
    test-fast)
        print_header "快速测试（不优化）"
        print_info "编译会更快，但证明生成可能较慢"
        echo ""
        RUST_LOG=info cargo test -- --nocapture
        print_success "测试通过！"
        ;;
    
    build)
        print_header "编译 Guest 程序"
        print_info "编译投票机的 RISC-V 程序"
        echo ""
        cd methods
        cargo build
        print_success "编译完成！"
        ;;
    
    check)
        print_header "检查代码质量"
        print_info "运行 cargo check（快速）"
        echo ""
        cargo check
        print_success "代码检查完成！"
        ;;
    
    clean)
        print_header "清理构建文件"
        print_info "删除 target 目录..."
        echo ""
        cargo clean
        print_success "清理完成！"
        ;;
    
    doc)
        print_header "打开文档"
        print_info "可用的文档文件："
        echo ""
        echo "  1. LEARNING_GUIDE.md"
        echo "     ➜ Rust 基础语法 + 投票机详细讲解"
        echo ""
        echo "  2. EXECUTION_TRACE.md"
        echo "     ➜ 测试执行流程的动画式讲解"
        echo ""
        echo "  3. RUST_SYNTAX_CHEATSHEET.md"
        echo "     ➜ Rust 语法速查表"
        echo ""
        echo "  4. README.md（原始文档）"
        echo "     ➜ 投票机的官方说明"
        echo ""
        print_info "在你的编辑器中打开这些文件来学习："
        echo "  code LEARNING_GUIDE.md"
        echo "  code EXECUTION_TRACE.md"
        echo "  code RUST_SYNTAX_CHEATSHEET.md"
        ;;
    
    explain)
        print_header "代码讲解模式"
        print_info "这将逐步讲解代码的执行过程"
        echo ""
        
        echo "投票机示例的三层架构："
        echo ""
        echo "┌─────────────────────────────────────┐"
        echo "│ Core (core/src/lib.rs)              │"
        echo "│ - VotingMachineState 结构体         │"
        echo "│ - vote() 方法（核心业务逻辑）        │"
        echo "│ - 各种 Params/Result/Commit 结构体   │"
        echo "└─────────────────────────────────────┘"
        echo "            ↓ 被使用 ↓"
        echo "┌─────────────────────────────────────┐"
        echo "│ Guest (methods/guest/src/bin/*.rs)  │"
        echo "│ - init.rs：初始化投票机              │"
        echo "│ - submit.rs：提交选票                │"
        echo "│ - freeze.rs：冻结投票站              │"
        echo "│ 在隔离的 RISC-V 虚拟机中执行        │"
        echo "└─────────────────────────────────────┘"
        echo "            ↓ 生成证明 ↓"
        echo "┌─────────────────────────────────────┐"
        echo "│ Host (src/lib.rs)                   │"
        echo "│ - PollingStation 类：管理投票流程    │"
        echo "│ - init()：初始化并获得证明           │"
        echo "│ - submit()：提交选票并获得证明       │"
        echo "│ - freeze()：冻结并获得证明           │"
        echo "└─────────────────────────────────────┘"
        echo ""
        
        echo "执行流程："
        echo ""
        echo "  1️⃣  Host 创建初始状态"
        echo "     { polls_open: true, voter_bitfield: 0, count: 0 }"
        echo ""
        echo "  2️⃣  Host 调用 init() 初始化"
        echo "     ↓ 将状态写入 Guest"
        echo "     ↓ Guest 计算状态哈希，提交 journal"
        echo "     ↓ Host 得到 Receipt（包含证明）"
        echo ""
        echo "  3️⃣  Host 调用 submit(&ballot) 提交选票"
        echo "     ↓ 将状态和选票写入 Guest"
        echo "     ↓ Guest 在隔离环境执行 vote() 方法"
        echo "     ↓ Guest 计算新状态哈希，将新状态写回"
        echo "     ↓ Host 读取新状态，更新本地 state"
        echo "     ↓ Host 得到 Receipt（包含证明）"
        echo ""
        echo "  4️⃣  重复步骤 3 多次投票"
        echo ""
        echo "  5️⃣  Host 调用 freeze() 冻结投票站"
        echo "     ↓ 同上，但将 polls_open 设为 false"
        echo ""
        echo "  6️⃣  验证所有 Receipt"
        echo "     ↓ 调用 receipt.verify() 验证零知识证明"
        echo "     ↓ 从 journal 解码提交记录"
        echo ""
        
        echo "关键概念："
        echo ""
        echo "▪️  所有权（Ownership）"
        echo "    - 每个值在 Rust 中有唯一的所有者"
        echo "    - 可以借用（&）但不转移所有权"
        echo "    - 可变借用（&mut）可以修改值"
        echo ""
        echo "▪️  位图（Bitfield）"
        echo "    - 用 u32 的 32 个比特表示 32 个投票者"
        echo "    - 1 << voter 创建投票者对应的掩码"
        echo "    - bitfield & mask 检查是否已投票"
        echo "    - bitfield |= mask 标记已投票"
        echo ""
        echo "▪️  零知识证明（ZKP）"
        echo "    - 证明投票逻辑在隔离环境正确执行"
        echo "    - 无需公开完整的状态信息"
        echo "    - 任何人都可以验证证明"
        echo ""
        
        print_info "详细讲解请查看 LEARNING_GUIDE.md"
        ;;
    
    debug)
        print_header "调试模式"
        print_info "在调试模式下运行测试（更详细的输出）"
        echo ""
        
        # 检查是否安装了 lldb 或 gdb
        if command -v lldb &> /dev/null; then
            echo "使用 LLDB 调试器..."
            # 这是一个简单的例子，实际调试需要更多配置
            print_warning "需要手动配置 VS Code 的 launch.json"
        else
            echo "直接运行测试（带所有日志）..."
            RUST_LOG=trace cargo test -- --nocapture --test-threads=1
        fi
        ;;
    
    modify)
        print_header "修改并运行"
        print_info "尝试修改示例以理解它的工作原理"
        echo ""
        echo "建议的修改（按难度升序）："
        echo ""
        echo "1. 简单修改："
        echo "   - 编辑 src/lib.rs 中的 protocol() 函数"
        echo "   - 改变 ballot1-6 的 voter 或 vote_yes 值"
        echo "   - 改变断言的预期值"
        echo "   - 运行测试看结果如何改变"
        echo ""
        echo "2. 中等修改："
        echo "   - 改变初始状态（polls_open, voter_bitfield, count）"
        echo "   - 添加更多选票"
        echo "   - 修改冻结时机"
        echo ""
        echo "3. 高级修改："
        echo "   - 增加 voter_bitfield 的大小（改为 u64）"
        echo "   - 添加时间戳字段到 VotingMachineState"
        echo "   - 实现投票者身份验证"
        echo "   - 统计"否"票数"
        echo ""
        
        print_info "推荐流程："
        echo "  1. 打开 src/lib.rs（最后的 #[test] fn protocol() 函数）"
        echo "  2. 修改其中一个 ballot 的参数"
        echo "  3. 运行 cargo test --release"
        echo "  4. 观察结果和日志"
        echo "  5. 重复，直到理解了为什么会这样"
        ;;
    
    analyze)
        print_header "代码分析"
        print_info "分析投票机示例的代码结构"
        echo ""
        
        print_info "文件结构："
        tree -L 2 -I 'target' .
        
        echo ""
        print_info "代码行数统计："
        echo "Core 层："
        wc -l core/src/lib.rs | awk '{print "  core/src/lib.rs: " $1 " 行"}'
        
        echo "Methods 层："
        wc -l methods/src/lib.rs 2>/dev/null | awk '{print "  methods/src/lib.rs: " $1 " 行"}' || echo "  (自动生成)"
        
        echo "Guest 层："
        find methods/guest/src -name "*.rs" -type f | xargs wc -l | tail -1 | awk '{print "  总计: " $1 " 行"}'
        
        echo "Host 层："
        wc -l src/lib.rs | awk '{print "  src/lib.rs: " $1 " 行"}'
        
        echo ""
        print_info "依赖分析："
        echo "主要依赖："
        grep -A 20 "^\[dependencies\]" Cargo.toml | grep -v "^\[" | head -10
        ;;
    
    help|--help|-h)
        print_header "投票机示例 - 快速开始指南"
        echo ""
        echo "用法：bash quick_start.sh [命令]"
        echo ""
        echo "可用命令："
        echo ""
        echo "  test           运行完整的测试（推荐首次运行）"
        echo "  test-fast      不优化编译，更快的运行"
        echo "  build          仅编译 Guest 程序"
        echo "  check          快速代码检查"
        echo "  clean          清理所有构建文件"
        echo "  doc            显示可用文档列表"
        echo "  explain        打印架构和执行流程图"
        echo "  debug          调试模式运行"
        echo "  modify         显示推荐的修改示例"
        echo "  analyze        分析代码结构和行数"
        echo "  help           显示此帮助信息"
        echo ""
        echo "首次使用建议："
        echo ""
        echo "  1. bash quick_start.sh doc      # 查看文档列表"
        echo "  2. bash quick_start.sh explain  # 了解架构"
        echo "  3. bash quick_start.sh test     # 运行测试"
        echo "  4. 打开 LEARNING_GUIDE.md       # 详细学习"
        echo ""
        ;;
    
    *)
        print_warning "未知命令: $CMD"
        echo ""
        echo "运行 'bash quick_start.sh help' 获取帮助"
        exit 1
        ;;
esac
