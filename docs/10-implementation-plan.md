# 逐步实现计划 — 当前状态与后续

> 最后更新: Phase A/B/C/D 完成, E/G/H 基础完成

## 已完成

| Phase | 项目 | 状态 |
|-------|------|------|
| **A** | F2 闭包 `-> Type` | ✅ |
| | F3/F4 多字节UTF-8 (`'🦀'`) | ✅ |
| | F5 `*expr = value` 解引用赋值 | ✅ |
| | const 作用域 | ✅ |
| **B** | M9 隐式return + parse_block hang修复 | ✅ |
| | F6 元组类型标注解构 | ✅ |
| | M1 `if let` (allow_struct_literal fix) | ✅ |
| **C** | M2 `impl` + `&self` 方法 | ✅ |
| | M6 String类型 + String::from | ✅ |
| | M5 枚举定义 + 构造函数 | ✅ |
| **D** | Move语义 + use-after-move | ✅ |
| | &T / &mut T 借用冲突检测 | ✅ |
| **I** | ariadne 彩色错误诊断 | ✅ |
| **F** | mod 文件解析 + flatten + pub | ✅ |
| **E** | 泛型单态化 `id<T>` | ✅ |
| **G** | ? 操作符 + Ok/Err/Some/None | ✅ |
| **H** | 闭包解析 + typeck | ✅ |

## 待实现 (按优先级)

### P1 — 核心语法

| # | 项目 | 难度 | 说明 |
|---|------|------|------|
| P1-1 | 数组 `.len()` / `.iter()` / `for..in` | 中 | 需要内置方法 + Iterator trait 或简化实现 |
| P1-2 | 枚举 match 解构 | 高 | tagged union tag extract → 模式变量绑定 |
| P1-3 | 泛型约束 `T: PartialOrd` | 高 | trait bound 语法 + typeck约束检查 |

### P2 — 类型系统

| # | 项目 | 难度 | 说明 |
|---|------|------|------|
| P2-1 | trait 定义 + 实现 | 高 | trait系统全链路 |
| P2-2 | 闭包 codegen | 高 | 环境捕获传递 + 匿名函数生成 |
| P2-3 | `Vec<T>` 类型 | 中 | 动态数组基础 |

### P3 — 高级特性

| # | 项目 | 难度 | 说明 |
|---|------|------|------|
| P3-1 | `dyn Trait` 语法 | 高 | 虚函数表 |
| P3-2 | `where` 子句 | 中 | 泛型约束替代语法 |
| P3-3 | `#[derive(...)]` | 中 | 内置derive |
| P3-4 | 对象文件输出 | 中 | LLVM 后端链接 |
| P3-5 | 前端教学平台 | 高 | Vue3 + Monaco Editor |

## 执行建议

优先按 P1-1 → P1-2 → P1-3 顺序，每个项目：
1. 创建测试用例 (对照rustc正确行为)
2. 实现功能
3. `bash scripts/test-suite.sh --rustc-verify` 验证无回归
4. git commit + push
