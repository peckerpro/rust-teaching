#!/usr/bin/env bash
set -euo pipefail

echo "=== Rust 教学编译器 - 开发环境配置 ==="

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

check() {
    if command -v "$1" &>/dev/null; then
        echo -e "${GREEN}[OK]${NC} $1"
        return 0
    else
        echo -e "${RED}[MISS]${NC} $1"
        return 1
    fi
}

echo ""
echo "1. 检查系统工具..."
check gcc || sudo apt install -y build-essential
check curl || sudo apt install -y curl
check git || sudo apt install -y git
check pkg-config || sudo apt install -y pkg-config

echo ""
echo "2. 检查Rust工具链..."
if check rustc; then
    echo "   Rust版本: $(rustc --version)"
    rustup component add rustfmt clippy rust-analyzer 2>/dev/null || true
else
    echo "   安装Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
    rustup component add rustfmt clippy rust-analyzer
fi

echo ""
echo "3. 检查LLVM..."
if check llvm-config-18; then
    echo "   LLVM版本: $(llvm-config-18 --version)"
elif check llvm-config; then
    echo "   LLVM版本: $(llvm-config --version)"
else
    echo -e "${YELLOW}[TODO]${NC} 手动安装LLVM 18:"
    echo "   wget https://apt.llvm.org/llvm.sh"
    echo "   chmod +x llvm.sh"
    echo "   sudo ./llvm.sh 18"
    echo "   sudo apt install -y libllvm18 llvm-18-dev libclang-18-dev"
fi

echo ""
echo "4. 检查其它依赖..."
check node || {
    echo "   安装Node.js 22..."
    curl -fsSL https://deb.nodesource.com/setup_22.x | sudo -E bash -
    sudo apt install -y nodejs
}
check docker || echo -e "${YELLOW}[TODO]${NC} 手动安装Docker: curl -fsSL https://get.docker.com | sudo sh"

echo ""
echo "5. 安装Rust开发工具..."
cargo install cargo-watch cargo-edit 2>/dev/null || true

echo ""
echo "6. 设置环境变量..."
cat >> "$HOME/.cargo/config.toml" <<'EOF' 2>/dev/null || true

# 内存优化：限制并行编译
[build]
jobs = 2
EOF

echo ""
echo "=== 环境检查完成 ==="
echo ""
echo "运行 'cargo build' 构建项目"
echo "运行 'cargo test' 执行测试"
