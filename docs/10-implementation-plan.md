# 逐步实现计划 — 基于外部验证 + 功能矩阵

> 优先级定义: P0=阻塞外部示例, P1=核心语法缺失, P2=标准库/方法支持, P3=高级特性

## 修复清单 (来自 rust-by-example 验证)

| # | 文件 | 错误 | 根因 | 修复难度 |
|---|------|------|------|----------|
| F1 | closures | hang (timeout) | `where F: Fn(i32) -> i32` 导致 parse_generics 死循环 | 中 |
| F2 | closures | `-> i32` syntax | ✅ 已修复 | - |
| F3 | scalar-types | `'ℤ'` panic | lexer_slice 多字节 char 边界 | 低 |
| F4 | constants | `'🦀'` panic | 同上 | 低 |
| F5 | ownership | `*s5 = value` | 解引用赋值语法不支持 | 中 |
| F6 | tuples | type mismatch | `let (x,y,z): (i32,f64,u8) = tup` 类型标注解构 | 中 |
| F7 | loops | `for e in array` | `for..in` 需迭代器 trait | 高 |
| F8 | arrays | `.len()`/`.iter()` | 标准库方法 | 高 |
| F9 | enums | match enum | 枚举变体模式匹配+构造函数 | 高 |
| F10 | structs | `impl` + `self` | impl 块方法调度 | 高 |
| F11 | traits | trait 系统 | trait 定义+实现+约束 | 很高 |
| F12 | generics | 泛型约束 | `T: PartialOrd` trait bound | 很高 |

## 功能矩阵未实现部分

| # | 特性 | 当前状态 | 需实现 |
|---|------|----------|--------|
| M1 | `if let` / `while let` | parser不支持 | parser: `if let Pat = expr { }` + 解糖为 match |
| M2 | `impl` + `self` 方法 | parser解析但忽略 | typeck: self参数注入, codegen: 方法调用生成 |
| M3 | trait 系统 | parser解析 | typeck: trait定义/实现/约束检查 |
| M4 | 泛型约束 `T: Trait` | 不支持 | parser: 冒号后类型列表, typeck: 约束检查 |
| M5 | 枚举完整支持 | 仅解析 | codegen: tagged union, match解构 |
| M6 | `String` 类型 | 未定义 | 内置String类型 (简化: 字符数组) |
| M7 | `Vec<T>` 类型 | 未定义 | 内置Vec类型 |
| M8 | `for..in` 迭代器 | 占位实现 | Iterator trait + for展开 |
| M9 | 隐式 return | 表达式块尾返回 | typeck: 块尾表达式作为返回类型 |
| M10 | 闭包 codegen | 仅typeck | 环境捕获传递, 匿名函数生成 |
| M11 | 属性 `#[derive]` | lexer跳过 | 内置derive实现 (Debug/Clone/Default) |

## 执行计划 (按优先级)

### Phase A: 快速修复 (预计 2h)
- F3/F4: 修复 lexer_slice 多字节返回空字符串
- F5: `*expr = value` 解引用赋值解析
- F1: 修复 closures hang (parse_generics where子句提前break)

### Phase B: 核心语法 (预计 4h)
- M1: `if let` 表达式
- M9: 隐式 return (块尾表达式)
- F6: 元组类型标注解构

### Phase C: 类型系统增强 (预计 6h)
- M2: `impl` + `self` 方法
- M6: `String` 内置类型
- M5: 枚举 match 解构

### Phase D: 高级特性 (预计 8h)
- M4/M8: 泛型约束 + Iterator
- M10: 闭包 codegen
- F7/F8: for..in 迭代 + 数组方法

## 开始执行 Phase A
