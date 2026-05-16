# Rust 教学编译器 — 技术文档

> 版本: v0.3.0 · 仓库: github.com/peckerpro/rust-teaching · 测试: 128/128 全通过

## 1. 系统架构

```
Source Code (.rs)
    │
    ▼
┌─────────────┐     ┌──────────┐     ┌───────────────┐     ┌─────────────┐
│   Lexer      │────▶│  Parser   │────▶│  Semantic     │────▶│  Codegen     │────▶ LLVM IR
│  649 行      │     │  1444 行   │     │  1199 行       │     │  662 行       │
└─────────────┘     └──────────┘     └───────────────┘     └─────────────┘
     │                    │                   │                    │
     ▼                    ▼                   ▼                    ▼
 Token Stream         AST + Span        HIR (typed AST)       LLVM Module
  (87种Token)         (26种Expr)        + Ownership/Borrow     (.ll)

Driver (150行): 编译流水线 + 模块文件解析 + items flatten
CLI (215行): gcc风格命令 + ariadne彩色错误
总计: ~5553行 Rust 代码, 8 crate
```

## 2. 模块依赖

```
rt-cli → rt-driver → rt-codegen → inkwell (LLVM)
              │            │
              ▼            ▼
        rt-semantic     rt-ast → rt-common
              │
              ▼
        rt-parser → rt-lexer → rt-common
```

## 3. 各 Crate 职责

| Crate | 行数 | 核心类型/功能 |
|-------|------|--------------|
| `rt-common` | 520 | `Span`, `Token(87种)`, `DiagnosticBag`, `SourceFile`, `Interner` |
| `rt-lexer` | 649 | `Lexer` (Iterator), `Cursor`, 属性跳过, 多字节UTF-8支持 |
| `rt-parser` | 1444 | `Parser` (递归下降+Pratt 14级), 10种Item, `parse_fn_param(&self简写)`, `parse_if_expr(if let)` |
| `rt-ast` | 714 | 26种Expr, 10种Item, 8种Pattern, 8种Ty, ClosureExpr(ret_ty), TryExpr |
| `rt-semantic` | 1199 | `NameResolver`(两遍), `TypeChecker`(推断+Move/Borrow), `Scope`, `SemTy`(31变体含String) |
| `rt-codegen` | 662 | LLVM IR生成: 控制流(phi), struct/enum, 泛型单态化, 方法调用 |
| `rt-driver` | 150 | `Session::compile()`, 模块文件解析, `flatten_items` |
| `rt-cli` | 215 | 12个CLI flags, `ariadne` 彩色诊断 |

## 4. 语言特性支持矩阵

### 4.1 解析 → 类型检查 → 代码生成

| 特性 | 解析 | Typeck | Codegen |
|------|:--:|:-----:|:-----:|
| 基础类型 (i8-u128, f32/64, bool, char, String) | ✅ | ✅ | ✅ |
| 字面量 (整数/浮点/布尔/字符/字符串/字节) | ✅ | ✅ | ✅ |
| 变量 (`let`, `let mut`, shadowing) | ✅ | ✅ | ✅ |
| 算术/比较/逻辑运算 | ✅ | ✅ | ✅(部分) |
| `if` / `else if` / `else` | ✅ | ✅ | ✅(phi) |
| `loop` / `while` / `for` | ✅ | ✅ | ✅ |
| `break` / `continue` | ✅ | ✅ | ✅ |
| `return` (显式+隐式) | ✅ | ✅ | ✅ |
| 函数 `fn` | ✅ | ✅ | ✅ |
| 嵌套函数 | ✅ | ✅ | ✅ |
| 闭包 `\|x\| expr` | ✅ | ✅ | ⬜ |
| 结构体 `struct` | ✅ | ✅ | ✅(字段+Lit) |
| `impl` + `&self` 方法 | ✅ | ✅ | ✅ |
| 元组 `(a, b)` + 解构 | ✅ | ✅ | ✅ |
| 数组 `[a, b]` / `[v; n]` | ✅ | ⬜ | ⬜ |
| 枚举 `enum` 定义+构造 | ✅ | ✅ | ✅ |
| 枚举 match 解构 | ⬜ | ⬜ | ⬜ |
| `match` 表达式 | ✅ | ✅(literal+wildcard) | ✅ |
| `if let` | ✅ | ⚠️(enum pattern待) | ⚠️ |
| `impl` + `self` 方法 | ✅ | ✅ | ✅ |
| 泛型 `<T>` | ✅ | ✅ | ✅(单态化) |
| 模块 `mod` / `pub` | ✅ | ⬜ | ✅ |
| Move 语义 | — | ✅ | — |
| 借用 `&T` / `&mut T` | — | ✅ | — |
| `println!` 宏 | ✅ | ✅ | ⬜ |
| `?` 操作符 | ✅ | ✅ | ✅ |
| `String::from()` | ✅ | ✅ | ✅ |
| `Option::Some()` | ✅ | ✅ | ✅ |

**图例**: ✅ 完成, ⚠️ 部分, ⬜ 未实现, — 不适用

### 4.2 语义类型系统 (SemTy, 31种变体)

```
I8 I16 I32 I64 I128 | U8 U16 U32 U64 U128 | ISize USize
F32 F64 | Bool | Char | Str | String | Unit | Never | Infer
Fn(FnTy) | Struct(StructTy) | Enum(EnumTy)
Ref(RefTy) | Tuple(Vec<SemTy>) | Array(Box<SemTy>, usize) | Slice(Box<SemTy>)
Generic(String)
```

### 4.3 LLVM 类型映射

| SemTy | LLVM Type |
|-------|-----------|
| I8/U8 | i8 |
| I16/U16 | i16 |
| I32/U32 | i32 |
| I64/U64/ISize/USize | i64 |
| F32 | float |
| F64 | double |
| Bool | i1 |
| Char | i8 |
| Str/String | i8* |
| Unit | {} |
| Ref(T) | T* |

## 5. 控制流代码生成

```
If/Else:  cond → then_bb / else_bb → merge_bb (phi node)
Loop:     loop_hdr → loop_body → loop_hdr (break → loop_exit)
While:    while_cond → while_body → while_cond (break → while_exit)
For:      for_cond(0..10) → for_body → for_inc → for_cond
Break:    unconditional branch to loop_stack.top().exit
Continue: unconditional branch to loop_stack.top().continue
```

## 6. 泛型单态化

```
fn id<T>(x: T) -> T { return x; }

调用 id(42)  → 检测 IntValue  → 生成 id_i32(i32) -> i32
调用 id(3.14) → 检测 FloatValue → 生成 id_f64(double) -> double
```

## 7. 运行时 (所有权/借用)

- **Move**: 非Copy类型赋值/传参后原变量失效, use-after-move 检测
- **Copy**: 基础整型/浮点/bool/char/Unit 自动Copy
- **&T**: 共享借用(可多个), 冲突检测
- **&mut T**: 可变借用(独占), 共享/可变冲突检测

## 8. 测试基础设施

```
tests/
├── positive/          (22)  基本正例
├── phase_a1/          (21)  扩展类型
├── phase_b/           (13)  控制流
├── phase_c/           (12)  复合类型
├── phase_d/           (9)   所有权借用
├── phase_e/           (3)   泛型
├── phase_g/           (3)   错误处理
├── phase_h/           (4)   闭包
├── negative/          (5)   反例
└── ir_compare/        (3)   rustc对比
总计: 97个 .rs 文件, 128测试用例
```

```
$ bash scripts/test-suite.sh --rustc-verify
Results: 128 passed, 0 failed, 0 skipped
```
