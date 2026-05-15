# RTC vs RUSTC — 功能差距分析报告

> 对比源: `jpbruinsslot/rust-by-example` (26 stars, Rust 教学示例)
> 测试文件: hello-world, variables, scalar-types, tuples, arrays, if-else, functions, structs, ownership, closures, traits, generics, error-handling, modules
> 测试方式: `rtc --check` vs `rustc --edition 2021 --crate-type bin`

## 总结

14 个测试文件，**全部被 rtc 拒绝**。rustc 通过 11/14（3 个因 `fn main()` 不存在而拒绝）。

## 按优先级排序的差距清单

### P0 — 阻塞性（几乎所有示例失败的根本原因）

| # | 差距 | 影响文件 | 说明 |
|---|------|----------|------|
| 1 | **`println!` 宏** | 14/14 | 所有示例都使用 `println!`。没有宏系统，任何实际 Rust 代码都无法编译 |
| 2 | **`#![allow(unused)]` 属性** | structs, ownership | 属性语法 `#[...]` 未被词法分析器处理，导致整个文件解析失败 |
| 3 | **`use std::...` 导入** | generics, error-handling | 外部 crate 导入语句未被 resolve，导致后续类型无法识别 |

### P1 — 关键缺失（教学核心功能）

| # | 差距 | 示例 | 说明 |
|---|------|------|------|
| 4 | **嵌套函数** | functions.rs | `fn outer() { fn inner() {} }` — 内部函数定义不支持 |
| 5 | **元组解构模式** | tuples.rs | `let (x, y, z) = tup;` — pattern 不支持元组解构 |
| 6 | **`impl` 块 + `self` 方法** | structs.rs | `impl Rectangle { fn area(self) -> u32 { ... } }` — 不支持 struct 方法 |
| 7 | **`if let` 表达式** | if-else.rs | `if let Some(value) = result { ... }` — 条件模式匹配不支持 |
| 8 | **数组重复语法** | arrays.rs | `[3; 5]` — 解析器不支持 `[expr; count]` 语法 |
| 9 | **泛型约束** | generics.rs | `fn largest<T: PartialOrd>(...)` — trait bound 语法不支持 |
| 10 | **Trait 定义 + 实现** | traits.rs | `trait Shape { fn area(&self) -> f64; }` — trait 系统未实现 |

### P2 — 标准库依赖（Teaching Compiler 可选择性实现）

| # | 差距 | 示例 | 说明 |
|---|------|------|------|
| 11 | **`String` 类型** | ownership.rs | `String::from("hello")`, `.len()`, `.push_str()` — 需要标准库或内置 String |
| 12 | **`Vec<T>` 类型** | generics.rs | `let numbers = vec![10, 20];` — 需要 `vec!` 宏和动态数组 |
| 13 | **切片 `&[T]`** | arrays.rs, generics.rs | `&arr[1..3]` — 切片类型需要完善 |
| 14 | **`Result<T, E>` 方法** | error-handling.rs | `.unwrap()`, `.ok()`, `.map()`, `.is_ok()` — Result 枚举方法 |
| 15 | **`Option<T>` 方法** | if-else.rs, generics.rs | `Option<T>` 模式匹配和构造 |

### P3 — 语法细节

| # | 差距 | 说明 |
|---|------|------|
| 16 | **闭包返回类型标注** | `\|a, b\| -> i32 { a + b }` — 闭包的 `-> Type` 语法 |
| 17 | **`dyn Trait` 语法** | traits.rs, error-handling.rs | `&dyn Shape`, `Box<dyn Error>` |
| 18 | **`where` 子句** | generics.rs | `where F: Fn(i32) -> i32` |
| 19 | **枚举泛型** | generics.rs | `enum HTTPResp<T> { Success(T), Error(T) }` |
| 20 | **`Derive` 宏** | structs.rs | `#[derive(Default)]` |
| 21 | **嵌套 `impl` 块** | structs.rs | 同一 struct 的多个 `impl` 块 |
| 22 | **`let else` 语法** | error-handling.rs | `let Ok(x) = f() else { return Err(...) };` |

## 建议教学编译器路线

对于教学用途，按以下优先级实现：

### 第 1 步：打通 "Hello World" (P0)
- `println!` 宏：先做死板实现，`println!("...")` 和 `println!("{}", expr)` 两种形式
- 属性跳过：`#![allow(unused)]` 在词法分析中跳过

### 第 2 步：核心语法补全 (P1)
- 嵌套函数：递归解析 fn item in block
- 元组解构：`let (a, b) = ...` 模式
- `impl` 块 + `self` 方法
- `if let` 表达式
- 数组重复语法 `[expr; count]`

### 第 3 步：类型系统增强 (P1+P2)
- String 内置类型
- Vec 内置类型
- `Result<T, E>` 枚举内置（含 `.unwrap()`）
- `Option<T>` 枚举内置
- `dyn Trait` 基础支持

### 第 4 步：高级特性 (P2+P3)
- 泛型约束 `T: PartialOrd`
- trait 定义 + 实现
- 闭包返回类型标注
