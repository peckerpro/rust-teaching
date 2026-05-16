# Rust 教学编译器 — 技术文档

> 版本: v0.2.0  
> 仓库: https://github.com/peckerpro/rust-teaching  
> 测试: 128/128 全通过 (rtc + rustc 交叉验证)

---

## 1. 系统架构

### 1.1 整体数据流

```
Source Code (.rs)
    │
    ▼
┌─────────────┐     ┌──────────┐     ┌───────────────┐     ┌─────────────┐
│   Lexer      │────▶│  Parser   │────▶│  Semantic     │────▶│  Codegen     │────▶ LLVM IR
│  (词法分析)   │     │ (语法分析)  │     │  (语义分析)    │     │  (代码生成)   │
└─────────────┘     └──────────┘     └───────────────┘     └─────────────┘
     │                    │                   │                    │
     ▼                    ▼                   ▼                    ▼
 Token Stream         AST + Span        HIR (typed AST)       LLVM Module
                                          + Diagnostics        (.ll / .o)
```

### 1.2 模块依赖关系

```
rt-cli ──▶ rt-driver ──▶ rt-codegen ──▶ inkwell (LLVM)
                │               │
                ▼               ▼
          rt-semantic      rt-ast ──▶ rt-common
                │
                ▼
          rt-parser ──▶ rt-lexer ──▶ rt-common
```

### 1.3 编译单元 (Crate) 职责

| Crate | 职责 | 行数 |
|-------|------|------|
| `rt-common` | Span(源码定位), Token(87种), Diagnostic(错误收集), Symbol(字符串池) | ~520 |
| `rt-lexer` | 手写状态机词法分析器, 支持注释/属性/字面量后缀 | ~614 |
| `rt-parser` | 递归下降 + Pratt 优先级表达式(14级), 支持10种 Item | ~1422 |
| `rt-ast` | AST节点定义(26种Expr + 10种Item + 8种Pattern + 8种Ty) | ~713 |
| `rt-semantic` | 名称解析(两遍), 类型推导/检查, 所有权Move, 借用冲突检测 | ~1093 |
| `rt-codegen` | LLVM IR生成(表达式/控制流/函数调用/struct/泛型单态化) | ~617 |
| `rt-driver` | 编译流水线编排 + 模块文件解析 + items flatten | ~150 |
| `rt-cli` | 命令行入口(12个flags) + ariadne彩色错误输出 | ~215 |

---

## 2. 词法分析 (Lexer)

### 2.1 设计

手写状态机, 一次遍历产出 Token 流。内部使用 `Cursor` 进行字符级迭代。

```
Cursor
  ├─ src: &str           // 源码引用
  ├─ pos: usize          // 当前字节位置
  ├─ peek() -> Option<u8>
  ├─ advance() -> Option<u8>
  ├─ eat_while(f)        // 条件消费
  └─ slice(lo, hi) -> &str
```

### 2.2 Token 分类

| 类别 | 数量 | 示例 |
|------|------|------|
| 关键字 | 33 | `fn`, `let`, `if`, `else`, `loop`, `while`, `for`, `match`, `struct`, `enum`, `impl`, `trait`, `mod`, `pub`, `use`, `return`, `break`, `continue` |
| 字面量 | 8 | `Integer`, `Float`, `Char`, `Byte`, `String`, `ByteString`, `Bool` |
| 运算符 | 24 | `+`, `-`, `*`, `/`, `%`, `==`, `!=`, `<`, `>`, `<=`, `>=`, `&&`, `\|\|`, `!`, `&`, `\|`, `<<`, `>>` |
| 复合赋值 | 10 | `+=`, `-=`, `*=`, `/=`, `%=`, `&=`, `\|=`, `^=`, `<<=`, `>>=` |
| 分隔符 | 12 | `(`, `)`, `{`, `}`, `[`, `]`, `,`, `;`, `:`, `::`, `->`, `=>` |

### 2.3 特殊处理

- **属性跳过**: `#[...]` 和 `#![...]` 被自动跳过
- **字面量后缀**: `42i64`, `3.14f32`, `0u8` 等类型标注被正确解析
- **点号上下文感知**: `t.0` (元组字段) vs `.5` (浮点字面量) — 通过检查前一个字节 (`prev_is_ident`) 区分
- **注释**: 行注释 `//`, 块注释 `/* */`, 文档注释 `///` 均正确跳过

---

## 3. 语法分析 (Parser)

### 3.1 设计

递归下降解析器, 表达式使用 Pratt 优先级爬升算法。

### 3.2 表达式优先级 (从低到高)

```
1  PREC_RANGE     .. / ..=
2  PREC_ASSIGN    = / += / -= / *= / ...
3  PREC_OR        ||
4  PREC_AND       &&
5  PREC_CMP       == / != / < / > / <= / >=
6  PREC_BITOR     |
7  PREC_BITXOR    ^
8  PREC_BITAND    &
9  PREC_SHIFT     << / >>
10 PREC_ADD       + / -
11 PREC_MUL       * / / / %
12 PREC_PREFIX    -x / !x / &x / *x  (前缀一元)
13 PREC_CALL      f(x) / arr[i]
13 PREC_FIELD     obj.field
```

### 3.3 后缀操作符 (Postfix)

| Token | Handler | 生成 |
|-------|---------|------|
| `(` | `parse_call` | `CallExpr` |
| `[` | `parse_index` | `IndexExpr` |
| `.` | `parse_field` | `FieldExpr` |
| `?` | `parse_try` | `TryExpr` |

### 3.4 解析流程

```
parse_program() ──循环──▶ parse_item()
                              │
                              ├─ fn    → parse_fn_item()
                              ├─ struct → parse_struct_item()
                              ├─ enum   → parse_enum_item()
                              ├─ impl   → parse_impl_item()
                              ├─ trait  → parse_trait_item()
                              ├─ mod    → parse_mod_item()
                              ├─ use    → parse_use_item()
                              ├─ const  → parse_const_item()
                              ├─ static → parse_static_item()
                              └─ type   → parse_type_alias_item()

parse_block() ──循环──▶  Stmt::Let / Stmt::Expr / Stmt::Item / Stmt::Semi
                              │
                              └─ tail expr (Option<Expr>)
```

---

## 4. 语义分析 (Semantic)

### 4.1 三阶段流水线

```
Phase 1: Name Resolution (NameResolver)
  ├─ declare_item() — 收集所有顶层声明到 global scope
  ├─ resolve_item() — 第二遍: 解析函数体中的名称引用
  └─ 输出: Scope (带类型信息的符号表)

Phase 2: Type Checking (TypeChecker)
  ├─ check_item()  — 遍历函数体, 推断表达式类型
  ├─ infer_expr()  — 类型推断引擎
  └─ 输出: Diagnostics (类型错误列表)

Phase 3: Borrow Checking (TypeChecker内联)
  ├─ moved_vars  — 移动语义跟踪
  ├─ borrow_state — 借用状态跟踪 (shared_count / has_mut)
  └─ 输出: use-after-move / borrow-conflict 错误
```

### 4.2 语义类型系统 (SemTy)

```rust
enum SemTy {
    I8, I16, I32, I64, I128,           // 有符号整数
    U8, U16, U32, U64, U128,           // 无符号整数
    ISize, USize,                       // 平台相关
    F32, F64,                           // 浮点
    Bool, Char, Str,                    // 基础类型
    Unit, Never, Infer,                 // 特殊类型
    Fn(Box<FnTy>),                      // 函数类型
    Struct(StructTy), Enum(EnumTy),     // 复合类型
    Ref(Box<RefTy>),                    // 引用
    Tuple(Vec<SemTy>),                  // 元组
    Array(Box<SemTy>, usize),           // 数组
    Slice(Box<SemTy>),                  // 切片
    Generic(String),                    // 泛型参数
}
```

### 4.3 所有权规则

| 类型 | Copy? | 说明 |
|------|-------|------|
| 所有整型 (i8-u128, isize, usize) | Yes | 栈上值, 赋值时自动复制 |
| f32, f64 | Yes | 浮点自动 Copy |
| bool, char | Yes | 基础类型 Copy |
| Unit, Never | Yes | 单元类型 Copy |
| struct, enum, tuple, String | No | 赋值时 Move, 原变量失效 |
| &T (共享引用) | Yes | 引用自身是 Copy 的 |

### 4.4 借用规则

- `&T`: 共享借用, 可多个同时存在
- `&mut T`: 可变借用, 独占, 不能与任何其他借出共存
- 变量有可变借出时, 不能直接使用该变量

---

## 5. 代码生成 (Codegen)

### 5.1 LLVM 类型映射

| SemTy | LLVM Type |
|-------|-----------|
| I8 / U8 | i8 |
| I16 / U16 | i16 |
| I32 / U32 | i32 |
| I64 / U64 / ISize / USize | i64 |
| I128 / U128 | i128 |
| F32 | float |
| F64 | double |
| Bool | i1 |
| Char | i8 |
| Unit | {} (空结构体) |
| Ref(T) | T* |

### 5.2 控制流代码生成

```
If/Else:    cond → then_bb / else_bb → merge_bb (phi node)
Loop:       loop_hdr → loop_body → loop_hdr (无限循环, break → loop_exit)
While:      while_cond (cond branch) → while_body → while_cond (break → while_exit)
For:        for_cond (0..10) → for_body → for_inc → for_cond (break → for_exit)
Break:      unconditional branch to exit_block (from loop_stack.top)
Continue:   unconditional branch to continue_block (from loop_stack.top)
```

### 5.3 泛型单态化

```
fn id<T>(x: T) -> T { return x; }

调用 id(42)  → 检测实参为 IntValue  → 生成 id_i32(i32) -> i32
调用 id(3.14) → 检测实参为 FloatValue → 生成 id_f64(double) -> double
```

`generic_templates: HashMap<String, FnItem>` 存储泛型函数 AST 模板。

---

## 6. 错误诊断

使用 `ariadne` 0.4 crate 实现类 rustc 的彩色错误输出:

```
Error: tests/negative/undefined_var.rs:1:20: cannot find value `x` in this scope
   ╭─[tests/negative/undefined_var.rs:1:1]
   │
 1 │ fn main() { return x; }
   │                    ┬
   │                    ╰── cannot find value `x` in this scope
───╯
```

---

## 7. 模块系统

- `mod foo;` — 自动从 `foo.rs` 或 `foo/mod.rs` 加载外部模块
- `mod foo { ... }` — 内联模块
- `pub` 关键字 — 解析器跳过, 不限制可见性 (教学编译器)
- 模块递归展平: `flatten_items()` 将嵌套模块展平为顶层 item 列表

---

## 8. 测试基础设施

### 8.1 测试目录结构

```
tests/
├── positive/          (22 files)  基本功能正例
├── phase_a1/          (21 files)  扩展类型系统
├── phase_b/           (13 files)  控制流
├── phase_c/           (12 files)  复合数据类型
├── phase_d/           (9 files)   所有权与借用
├── phase_e/           (3 files)   泛型
├── phase_g/           (3 files)   错误处理
├── phase_h/           (4 files)   闭包
├── negative/          (5 files)   核心反例
└── ir_compare/        (3 files)   rustc 交叉对比
```

### 8.2 测试运行

```bash
bash scripts/test-suite.sh               # 全部测试 (rtc only)
bash scripts/test-suite.sh --rustc-verify # rtc + rustc 交叉验证
```

### 8.3 测试结果

```
Results: 128 passed, 0 failed, 0 skipped
```
