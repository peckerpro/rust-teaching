# Rust 教学编译器 — 核心功能扩展路线图

> 当前版本 v0.1.0 支持：函数(含递归/互递归)、基础类型(i32/f64/bool)、算术/比较/逻辑运算、if/else、return、let 绑定、函数调用、类型检查、LLVM IR 生成。
> 以下按优先级排序，每个 Phase 对应一次可交付迭代。

---

## Phase A: 基础类型系统完善 (预计 2-3 天)

### A1. 完善基础类型支持
- [ ] 所有整数类型: `i8`, `i16`, `i64`, `u8`, `u16`, `u32`, `u64`, `isize`, `usize`
- [ ] 浮点类型: `f32`, `f64`
- [ ] `char` 类型完整支持
- [ ] 类型后缀字面量: `42i64`, `3.14f32`
- [ ] 类型转换 (typeck): 自动类型提升规则

### A2. 类型别名 type 语义
- [ ] `type Age = i32;` 在语义分析中替换为实际类型
- [ ] 支持泛型 type alias

### A3. 单元类型 `()` 完善
- [ ] `()` 作为表达式 (当前仅 AST 定义)
- [ ] 空 return 语句的 Unit 类型一致性

---

## Phase B: 控制流完善 (预计 3-4 天)

### B1. loop/while/for 代码生成
- [ ] `loop { break; }` — LLVM 基本块 + 无条件分支
- [ ] `break` 带值: `break expr;`
- [ ] `while cond { }` — 条件分支代码生成
- [ ] `for pat in iter { }` — 基础迭代器代码生成

### B2. match 表达式
- [ ] 枚举变体模式匹配
- [ ] 字面量模式匹配
- [ ] 通配符 `_` 模式
- [ ] match 穷尽性检查

### B3. continue 语句
- [ ] `continue` 跳转到循环头部
- [ ] 嵌套循环的 continue 语义

---

## Phase C: 复合数据类型 (预计 4-5 天)

### C1. 结构体 (完整)
- [ ] 类型检查: 字段访问 (`struct.field`)
- [ ] 代码生成: 结构体内存布局 (LLVM struct type)
- [ ] 结构体字面量: `Point { x: 1, y: 2 }`
- [ ] 结构体更新: `Point { x: 5, ..base }`
- [ ] 方法调用: `self` 参数、`impl` 块

### C2. 枚举 (完整)
- [ ] 代数据的枚举变体
- [ ] 枚举内存布局 (tagged union / LLVM 实现)
- [ ] 枚举构造函数: `Option::Some(42)`
- [ ] match 解构枚举

### C3. 元组
- [ ] 元组字面量: `(1, 2, 3)`
- [ ] 元组索引: `tuple.0`, `tuple.1`
- [ ] 元组解构: `let (x, y) = point;`

### C4. 数组 / 切片
- [ ] 数组字面量 `[1, 2, 3]` 和重复 `[0; 100]`
- [ ] 数组索引 `arr[i]` 代码生成 (带边界检查)
- [ ] 切片类型 `&[T]` 基础

---

## Phase D: 所有权与借用 (预计 5-7 天)

这是 Rust 最核心的特性，教学重点。

### D1. 所有权基础
- [ ] Move 语义: 变量赋值后原变量失效
- [ ] Copy trait: 基础类型自动 Copy
- [ ] 所有权转移在函数调用中的传递
- [ ] Drop: 作用域结束时清理

### D2. 借用检查
- [ ] `&T` 共享借用: 多个不可变引用
- [ ] `&mut T` 可变借用: 唯一可变引用
- [ ] NLL (Non-Lexical Lifetimes) 简化实现
- [ ] 借用规则错误诊断 (diagnostic messages)

### D3. 生命周期
- [ ] 生命周期标注: `fn foo<'a>(x: &'a i32) -> &'a i32`
- [ ] 生命周期省略规则 (3 条规则)
- [ ] 生命周期推断

### D4. 智能指针 (教学演示)
- [ ] `Box<T>`: 堆分配基础
- [ ] `Rc<T>`: 引用计数 (简化)
- [ ] `Arc<T>`: 原子引用计数 (简化)

---

## Phase E: 泛型系统 (预计 5-7 天)

### E1. 泛型函数
- [ ] `fn id<T>(x: T) -> T { x }`
- [ ] 泛型参数解析、名称解析、类型检查
- [ ] 单态化 (Monomorphization): 为每个具体类型生成 IR

### E2. 泛型结构体/枚举
- [ ] `struct Point<T> { x: T, y: T }`
- [ ] `enum Option<T> { Some(T), None }`
- [ ] `enum Result<T, E> { Ok(T), Err(E) }`

### E3. 泛型方法
- [ ] `impl<T> Point<T> { fn x(&self) -> &T }`
- [ ] 泛型约束: `T: PartialEq` (trait bounds)

### E4. 基础 Trait 系统
- [ ] trait 定义和实现
- [ ] trait 对象 (`dyn Trait`) 和虚函数表
- [ ] `#[derive(Debug, Clone, PartialEq)]` 手工实现
- [ ] `Copy`, `Clone`, `Drop` trait 语义

---

## Phase F: 模块与可见性 (预计 2-3 天)

### F1. 模块系统
- [ ] `mod my_module { }` 内联模块
- [ ] 文件模块: `mod my_module;` → `my_module.rs`
- [ ] 路径解析: `crate::`, `self::`, `super::`

### F2. 可见性
- [ ] `pub` 关键字语义
- [ ] `pub(crate)`, `pub(super)` 简化实现

### F3. use 语句完善
- [ ] `use std::collections::HashMap;`
- [ ] `use std::{fs, io};` 批量导入
- [ ] `use std::collections::*;` 通配符导入
- [ ] `as` 别名

---

## Phase G: 错误处理 (预计 2-3 天)

### G1. Result 类型
- [ ] `Result<T, E>` 标准库模拟
- [ ] `?` 操作符: 自动传播错误
- [ ] `unwrap()`, `expect()` 方法 (通过 trait)

### G2. panic! 宏
- [ ] `panic!("message")` 运行时错误
- [ ] 栈展开 (LLVM landing pads)

---

## Phase H: 高级特性 (预计 5-7 天)

### H1. 闭包
- [ ] 闭包语法: `|x, y| x + y`
- [ ] 环境捕获: 按引用/按值/move
- [ ] Fn/FnMut/FnOnce trait 体系 (简化)

### H2. 迭代器
- [ ] Iterator trait: `next()`, `map()`, `filter()`, `collect()`
- [ ] for 循环展开为迭代器调用

### H3. 模式匹配增强
- [ ] `if let`, `while let`
- [ ] 切片模式: `[first, rest @ ..]`
- [ ] 范围模式: `1..=5`

### H4. 声明宏 macro_rules!
- [ ] `macro_rules!` 基础解析
- [ ] 模式匹配和展开
- [ ] 常用宏: `vec!`, `println!`, `assert!`

### H5. 属性 (Attributes)
- [ ] `#[derive(...)]`
- [ ] `#[cfg(...)]` 条件编译
- [ ] `#[test]`

---

## Phase I: 编译器质量 (持续)

### I1. 错误诊断
- [ ] ariadne 集成: 带颜色/箭头的源码错误提示
- [ ] 错误恢复: 解析器跳过错误继续
- [ ] 多错误报告: 单次编译收集所有错误

### I2. 测试基础设施
- [ ] 对 rustc 输出进行自动化比对
- [ ] 编译正确性测试 (执行编译产物)
- [ ] 性能基准测试

### I3. 调试支持
- [ ] `--emit tokens` 词法调试
- [ ] `--emit ast` AST 可视化
- [ ] `--dump-scope` 符号表导出

### I4. 代码优化
- [ ] LLVM 优化 Pass 集成 (O0/O1/O2/O3)
- [ ] 常量折叠
- [ ] 死代码消除

---

## 参考架构

```
                    Source Code
                        │
                        ▼
┌─────────────────────────────────────────────┐
│  Lexer  (手写状态机)                          │
│  → Token Stream                             │
└─────────────────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────┐
│  Parser (递归下降 + Pratt 表达式)             │
│  → AST (带 Span)                            │
└─────────────────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────┐
│  Semantic Analysis (3-pass)                 │
│  1. Name Resolution (作用域/符号表)           │
│  2. Type Checking (类型推导/统一)              │
│  3. Borrow Checking (所有权/借用/生命周期)      │
│  → HIR (类型标注 AST)                        │
└─────────────────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────┐
│  Codegen (inkwell → LLVM IR)                │
│  → LLVM IR (.ll)                            │
└─────────────────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────┐
│  LLVM Backend                               │
│  → Object Code (.o) → Executable            │
└─────────────────────────────────────────────┘
```

---

## 编码约定

1. **教学优先**: 每个 Pass 的中间产物可导出 (tokens/AST/IR/SymbolTable)
2. **错误信息友好**: 所有编译错误包含行号、列号、上下文
3. **分阶段可运行**: 每个 Phase 完成后 `cargo test` 全部通过
4. **TDD**: 先写测试用例 (参考 rustc 正确行为)，再实现功能
