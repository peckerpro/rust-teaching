# Rust 教学编译器 — 使用说明

## 快速开始

```bash
# 设置环境
source ~/.cargo/env
export LLVM_SYS_180_PREFIX=$(llvm-config-18 --prefix)

# 编译
cd rust-teaching && cargo build -p rt-cli

# Hello World
echo 'fn main() { println!("Hello, world!"); }' > hello.rs
./target/debug/rtc hello.rs
```

## CLI 命令

```
rtc [OPTIONS] <INPUT>

Options:
  -o, --output <FILE>     输出文件名
  -c, --compile-only      仅编译到 .o (实际输出 LLVM IR)
  -S, --emit-ir           输出 LLVM IR
  -v, --verbose           显示编译各阶段
  -C, --check             仅类型检查
  --emit <tokens|ast|ir>  输出中间表示
  --parse-only            仅语法分析
  --lex-only              仅词法分析 (输出 Token 流)
```

## 支持的语言特性

### 基础类型和变量

```rust
fn main() {
    let a: i32 = 42;
    let b: f64 = 3.14;
    let c: bool = true;
    let d: char = 'A';
    let e: String = String::from("hello");
    let mut f: i32 = 1;
    f = f + 1;  // 赋值
    let g: i32 = f;  // i32 是 Copy 的
}
```

### 控制流

```rust
fn main() {
    // if/else
    let x: i32 = if 2 > 1 { 10 } else { 20 };

    // loop + break
    let mut i: i32 = 0;
    loop { i = i + 1; if i > 5 { break; } }

    // while + continue
    let mut j: i32 = 0;
    while j < 10 { j = j + 1; if j == 5 { continue; } }
}
```

### 函数

```rust
fn add(a: i32, b: i32) -> i32 { return a + b; }
fn max(a: i32, b: i32) -> i32 { if a > b { a } else { b } }  // 隐式return
fn swap(x: i32, y: i32) -> (i32, i32) { return (y, x); }

fn main() {
    let _s: i32 = add(3, 4);
    let (a, b) = swap(1, 2);  // 元组解构
}
```

### 结构体 + impl

```rust
struct Point { x: i32, y: i32 }

impl Point {
    fn new(x: i32, y: i32) -> Point { return Point { x, y }; }
    fn x_val(&self) -> i32 { return self.x; }
}

fn main() {
    let p: Point = Point::new(1, 2);
    let _v: i32 = p.x;
    let _x: i32 = x_val(p);
}
```

### 枚举 + match

```rust
enum Option { Some(i32), None }

fn main() {
    let x = Option::Some(42);
    let y: i32 = match 2 {
        1 => 10,
        2 => 20,
        _ => 30,
    };
}
```

### 所有权与借用

```rust
struct Val { v: i32 }

fn consume(x: Val) -> i32 { return x.v; }

fn main() {
    let a: Val = Val { v: 10 };
    let _r: i32 = consume(a);
    // let _x: i32 = a.v;    // ❌ use of moved value

    let x: i32 = 42;
    let y: i32 = x;           // ✅ i32 是 Copy
    let z: i32 = x;

    let r1: &i32 = &y;
    let r2: &i32 = &y;        // ✅ 多个共享借出
    // let r3: &mut i32 = &mut y;  // ❌ 已有共享借出
}
```

### 泛型

```rust
fn id<T>(x: T) -> T { return x; }

fn main() {
    let _a: i32 = id(42);
    let _b: f64 = id(3.14);
}
```

### 闭包

```rust
fn main() {
    let f = |x: i32, y: i32| x + y;
    let _r: i32 = f(1, 2);

    let g = || 42;
    let _s: i32 = g();

    let x: i32 = 10;
    let h = |y: i32| x + y;   // 环境捕获
}
```

### 错误处理

```rust
fn main() {
    let _x: i32 = Ok(42)?;
    let _a = Some(10);
    let _b: String = String::from("hello");
}
```

### LLVM IR 输出示例

```bash
$ rtc -S test.rs
```

```llvm
define i32 @add(i64 %0, i64 %1) {
entry:
  %add = add i64 %0, %1
  ret i64 %add
}

define {} @main() {
entry:
  %call = call i32 @add(i64 3, i64 4)
  ret void
}
```

### 错误诊断示例

```
Error: test.rs:1:23: mismatched types: expected i32, found bool
   ╭─[test.rs:1:1]
   │
 1 │ fn main() { let x: i32 = true; }
   │                       ───┬───
   │                          ╰───── mismatched types: expected i32, found bool
───╯
```

## 测试

```bash
bash scripts/test-suite.sh               # rtc only (92 tests)
bash scripts/test-suite.sh --rustc-verify # rtc + rustc (128 tests)
```
