# Rust 教学编译器 - 技术栈方案

## 1. 编译器核心

| 层次 | 技术选型 | 选型理由 |
|------|----------|----------|
| 实现语言 | **Rust** (edition 2024) | "吃自己的狗粮"教学价值；内存安全减少bug；cargo生态完整 |
| 编译器后端 | **LLVM 18** | 文档最丰富、社区最大、教学资源最多；与rustc同源便于对比学习 |
| LLVM绑定 | **inkwell** | 安全Rust封装，类型安全，API简洁，官方维护 |
| 词法分析 | **手写** | 教学目的：状态机、错误恢复，理解实现细节 |
| 语法分析 | **手写递归下降 + Pratt解析** | 教学目的：上下文敏感解析，比parser generator更易理解 |
| 类型检查 | **手写** | 基于Hindley-Milner简化版，结合Rust所有权语义 |
| 中间表示 | **LLVM IR** + 自定义教学IR | LLVM IR用于实际编译，自定义IR用于教学可视化 |

### 编译器架构（经典三阶段）
```
Source Code
    │
    ▼
┌─────────────────┐
│   Lexer          │  → Token Stream
│   (src/lexer/)    │
└─────────────────┘
    │
    ▼
┌─────────────────┐
│   Parser         │  → AST (带Span)
│   (src/parser/)   │
└─────────────────┘
    │
    ▼
┌─────────────────┐
│   Semantic       │  → HIR (类型标注AST)
│   (src/semantic/) │    Name Resolve → Type Check → Borrow Check
│  ├─ resolve.rs   │
│  ├─ typeck.rs    │
│  └─ borrowck.rs  │
└─────────────────┘
    │
    ▼
┌─────────────────┐
│   Codegen        │  → LLVM IR → Object Code
│   (src/codegen/) │
└─────────────────┘
```

### 编译单元 (Crate) 划分
```
crates/
├── common/          # 共享类型：Span, Token, Diagnostic, Symbol
├── lexer/           # 词法分析器
├── parser/          # 语法分析器
├── ast/             # AST节点定义 + 访问者trait
├── semantic/        # 语义分析(resolve + typeck + borrowck)
├── codegen/         # LLVM IR生成
├── driver/          # 编译器驱动(组合各阶段, 错误报告)
└── cli/             # 命令行入口
```

---

## 2. 后端服务（微服务）

| 层次 | 技术选型 | 版本 | 理由 |
|------|----------|------|------|
| Web框架 | **Axum** | 0.8+ | Rust生态最活跃、tokio原生、WebSocket支持好 |
| ORM | **SQLx** | 0.8+ | 编译时SQL检查、async、无DSL学习成本 |
| 数据库 | **PostgreSQL** | 16 | 成熟稳定、JSON支持、丰富索引类型 |
| 缓存 | **Redis** | 7.4+ | 编译结果缓存、会话存储、任务队列 |
| 对象存储 | **MinIO** | 最新 | S3兼容、轻量、自托管(替代云OSS) |
| 消息队列 | **SQLite** (轻量替代) | - | 在资源受限环境下，用SQLite代替RabbitMQ做任务队列表 |
| 容器化 | **Docker** + docker-compose | - | 环境一致性、沙箱编译 |
| 反向代理 | **Nginx** | 最新 | 静态资源、SSL终结、负载均衡 |
| API文档 | **utoipa** + Swagger UI | - | Rust原生OpenAPI生成 |

### 依赖精简策略（适配8GB RAM）
- 不使用重量级MQ(RabbitMQ/Redis Streams)，改用数据库轮询
- MinIO在测试环境可用本地文件系统替代
- 开发时PostgreSQL可用SQLite替代(通过SQLx兼容)

```
微服务技术栈摘要:
┌────────────────────────────────────────────┐
│ Compiler Service                            │
│ ├─ Axum HTTP Server                         │
│ ├─ compiler-core (workspace lib)            │
│ ├─ Docker SDK (bollard) for sandbox         │
│ └─ WebSocket for real-time progress         │
├────────────────────────────────────────────┤
│ Teaching Service                            │
│ ├─ Axum HTTP Server                         │
│ ├─ SQLx + PostgreSQL                        │
│ └─ File storage (local)                     │
├────────────────────────────────────────────┤
│ User Service                                │
│ ├─ Axum HTTP Server                         │
│ ├─ SQLx + PostgreSQL                        │
│ ├─ JWT (jsonwebtoken crate)                 │
│ └─ bcrypt (password hashing)                │
├────────────────────────────────────────────┤
│ API Gateway                                 │
│ └─ Nginx (reverse proxy + rate limit)       │
└────────────────────────────────────────────┘
```

---

## 3. 前端（教学平台）

| 层次 | 技术 | 理由 |
|------|------|------|
| 框架 | **Vue 3** + TypeScript | 学习曲线平缓、适合教学场景快速开发 |
| 构建工具 | **Vite** | 快速HMR、ESM原生 |
| UI组件库 | **Element Plus** | 成熟Vue3组件库、中文文档 |
| 代码编辑器 | **Monaco Editor** | VS Code同款、Rust语法高亮、错误标记 |
| AST可视化 | **D3.js** + **vis-network** | 树图/网络图/流程图 |
| CFG可视化 | **Mermaid.js** | 轻量、声明式流程图 |
| HTTP客户端 | **Axios** | 标准选择 |
| WebSocket | 原生WebSocket + reconnecting | 编译进度实时推送 |

---

## 4. DevOps & 基础设施

| 领域 | 工具 | 用途 |
|------|------|------|
| CI/CD | **GitHub Actions** | 自动测试、构建、镜像推送 |
| 代码质量 | **cargo clippy** + **cargo fmt** | Lint + 格式化 |
| 测试 | **cargo test** + **proptest** | 单元测试 + 属性测试 |
| 覆盖率 | **cargo-tarpaulin** | 代码覆盖率报告 |
| 日志 | **tracing** (Rust) | 结构化日志 |
| 监控 | **prometheus** + Grafana (可选) | 指标采集(资源受限时用日志代替) |

---

## 5. 开发环境配置 (VMware + Ubuntu 24.04)

### 资源分配
| 资源 | 配置 | 说明 |
|------|------|------|
| vCPU | 2×2 = 4核 | 编译时4核并行 |
| RAM | 8 GB | 需要内存优化策略 |
| Disk | 70 GB | OS(~15G) + 工具链(~10G) + 项目(~5G) + Docker镜像(~10G) 余量~30G |
| Swap | 4 GB | 编译大项目时的缓冲 |

### 安装清单
```bash
# 1. 系统基础
sudo apt update && sudo apt upgrade -y
sudo apt install -y build-essential curl wget git pkg-config libssl-dev

# 2. Rust 工具链
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# 选 stable-x86_64-unknown-linux-gnu
rustup component add rustfmt clippy rust-analyzer

# 3. LLVM 18
wget https://apt.llvm.org/llvm.sh
chmod +x llvm.sh
sudo ./llvm.sh 18
sudo apt install -y libllvm18 llvm-18-dev llvm-18-tools libclang-18-dev libpolly-18-dev

# 4. PostgreSQL 16
sudo apt install -y postgresql postgresql-client
sudo systemctl enable postgresql
sudo systemctl start postgresql

# 5. Redis
sudo apt install -y redis-server

# 6. Docker
curl -fsSL https://get.docker.com | sudo sh
sudo usermod -aG docker $USER
sudo systemctl enable docker

# 7. Node.js (前端)
curl -fsSL https://deb.nodesource.com/setup_22.x | sudo -E bash -
sudo apt install -y nodejs

# 8. 开发工具
sudo apt install -y htop neovim tmux
cargo install cargo-watch cargo-edit cargo-tarpaulin
```

### LLVM安装失败时的Fallback方案
```bash
# Fallback: 使用Ubuntu自带LLVM + llvm-sys
sudo apt install -y llvm-dev libclang-dev
# 在Cargo.toml中使用:
# llvm-sys = "180" (自动检测系统LLVM)
# 而非 inkwell (inkwell依赖llvm-sys)
```

### 内存优化策略
| 策略 | 方法 |
|------|------|
| 编译并行度 | `export CARGO_BUILD_JOBS=2` 限制编译并行度 |
| 增量编译 | `CARGO_INCREMENTAL=1` 保留增量编译缓存 |
| Swap | 4GB swap作为内存溢出缓冲区 |
| Docker | 限制容器内存 `--memory=512m` |
| IDE | 使用neovim而非VSCode节省内存 |
| 前端 | 开发时Vite按需编译，不build |

---

## 6. 参考开源项目对比

| 项目 | 语言 | 后端 | 特点 | 参考价值 |
|------|------|------|------|----------|
| **rustc** | Rust | LLVM | 完整实现，架构复杂 | 架构设计、AST定义、HIR/MIR设计 |
| **gccrs** | C++ | GCC | GCC后端集成 | GCC IR生成思路 |
| **rustc_codegen_gcc** | Rust | GCC(libgccjit) | Rust前端+GCC后端 | 后端抽象层设计 |
| **mrustc** | C++ | C transpile | 极简实现 | 小型编译器设计哲学 |
| **Crafting Interpreters** | C/Java | 解释器 | 经典教材 | Lexer/Parser教学设计 |
| **Writing a C Compiler** | 多语言 | x86 asm | 从零构建 | 全流程参考 |

### 本项目定位
- **教学优先**：代码清晰 > 性能
- **分阶段**：MVP → 扩展 → 高级特性
- **可观测**：每个编译阶段产物可视化
- **轻量**：适配8GB RAM虚拟机
