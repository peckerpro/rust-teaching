# Rust 教学编译器 — 项目概览

> **用于下一个 AI Session 的快速上下文文档**

## 基本信息

| 项目 | 值 |
|------|-----|
| 仓库 | `github.com/peckerpro/rust-teaching` |
| 版本 | v0.3.0 |
| 语言 | Rust (edition 2024), 目标: LLVM IR |
| 代码规模 | 5,553行, 8 crate, 35源文件 |
| 测试 | 128/128 全通过 (rtc + rustc交叉验证) |
| 外部验证 | 11/29 通过 (rust-by-example) |
| 外部依赖 | inkwell(LLVM,保留), ariadne(诊断,保留), clap/thiserror/serde/tracing(待自研) |

## 项目定位

面向教学的 Rust 语言编译器。核心目标:
1. **教学演示**: 展示词法→语法→语义→代码生成全流程
2. **可观测性**: 每阶段中间产物可导出 (`--lex-only`, `--parse-only`, `--emit ir`)
3. **功能覆盖**: 实现 Rust 核心子集
4. **工程实践**: TDD, 模块化, git版本控制

## 架构速览

```
Source Code → Lexer(状态机, 87种Token) → Parser(递归下降+Pratt14级)
    → AST → Resolver(两遍名称解析) → TypeChecker(推断+Move+Borrow)
    → Codegen(inkwell→LLVM IR) → 输出 .ll
```

```
crates/: common → lexer → parser → ast → semantic → codegen → driver → cli
tests/:  positive/(21) phase_a1/(21) phase_b/(13) phase_c/(12)
        phase_d/(9) phase_e/(3) phase_g/(3) phase_h/(4)
        negative/(5) ir_compare/(3)
```

## 关键文件

| 文件 | 作用 |
|------|------|
| `Cargo.toml` | workspace定义, 版本0.1.0, edition 2024 |
| `scripts/test-suite.sh` | 测试入口 (--rustc-verify for cross-check) |
| `crates/parser/src/parser.rs` | 1444行, 核心解析器 |
| `crates/semantic/src/typeck.rs` | 528行, 类型检查+所有权/借用 |
| `crates/codegen/src/codegen.rs` | 563行, LLVM IR生成 |
| `crates/lexer/src/lexer.rs` | 540行, 词法分析器 |
| `docs/07-technical-doc.md` | 最新技术文档 |
| `docs/09-completion-report.md` | 完成情况+功能矩阵 |
| `docs/10-implementation-plan.md` | 待实现项目清单 |

## 已实现核心功能

- **类型系统**: 31种SemTy (i8-u128, f32/64, bool, char, str, String, 复合类型)
- **控制流**: if/else(phi), loop/while/for, break/continue
- **函数**: fn定义, 嵌套函数, 递归, 隐式return
- **闭包**: `|x| body`, `|x| -> T { body }`, `|| body`, 环境捕获(typeck)
- **struct**: 字段访问, 字面量, `impl` + `&self` 方法
- **enum**: 定义, 构造函数 (`Option::Some(42)`)
- **match**: 字面量+通配符模式
- **if let**: 解糖为match
- **泛型**: `<T>` + 调用点单态化 (`id_i32`, `id_f64`)
- **所有权**: Move语义, use-after-move检测
- **借用**: `&T`共享, `&mut T`独占, 冲突检测
- **模块**: `mod foo;` → `foo.rs` 文件加载
- **错误处理**: `?` 操作符, Ok/Err/Some/None
- **字符串**: String类型, String::from()
- **诊断**: ariadne彩色错误输出
- **CLI**: gcc风格 (-S, -c, -o, --check, --lex-only, --emit)

## 待实现 (P1优先级)

1. **数组方法**: `.len()` / `.iter()` / `for..in` 迭代
2. **枚举 match 解构**: tagged union tag extract → 变体模式变量绑定
3. **泛型约束**: `T: PartialOrd` trait bound

## 常见操作

```bash
# 环境设置
source ~/.cargo/env
export LLVM_SYS_180_PREFIX=$(llvm-config-18 --prefix)

# 编译
cargo build -p rt-cli

# 测试
cargo test -p rt-lexer   # 单crate测试
bash scripts/test-suite.sh               # 全部rtc测试
bash scripts/test-suite.sh --rustc-verify # +rustc交叉验证

# 运行
./target/debug/rtc --check test.rs       # 仅类型检查
./target/debug/rtc -S test.rs            # 输出LLVM IR
./target/debug/rtc --lex-only test.rs    # 输出Token流

# git
git add -A && git commit -m "..." && git -c http.version=HTTP/1.1 push origin main
```

## 已知问题

1. 闭包codegen: 环境捕获未生成LLVM函数 (typeck已完成)
2. 枚举match解构: 仅支持字面量/通配符，不支持变体模式
3. 隐式return with shared-brace: 复杂嵌套情况可能hang (已通过peek_tok().is_none()缓解)
4. 浮点 `!=` `<` `>` 比较: codegen未实现
5. `&&` / `||`: codegen未实现短路求值
6. 对象文件输出: 仅生成LLVM IR (.ll)

## 参考项目

- [rust-lang/rust](https://github.com/rust-lang/rust) — 官方编译器 (架构参考)
- [Rust-GCC/gccrs](https://github.com/Rust-GCC/gccrs) — GCC后端
- [rust-lang/rustc_codegen_gcc](https://github.com/rust-lang/rustc_codegen_gcc) — 后端抽象层
- [jpbruinsslot/rust-by-example](https://github.com/jpbruinsslot/rust-by-example) — 外部验证测试集
