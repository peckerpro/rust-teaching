# Rust 教学编译器 — 使用说明

## 快速开始

### 编译

```bash
cd rust-teaching
source ~/.cargo/env
export LLVM_SYS_180_PREFIX=$(llvm-config-18 --prefix)
cargo build -p rt-cli
```

### Hello World

```bash
echo 'fn main() { println!("Hello, world!"); }' > hello.rs
./target/debug/rtc hello.rs
```

### 常用命令

```bash
rtc main.rs                    # 编译到 LLVM IR (stdout)
rtc main.rs -S                 # 输出 LLVM IR
rtc main.rs -o output.ll       # 输出到文件
rtc main.rs --check            # 仅类型检查, 不生成代码
rtc main.rs --parse-only       # 仅语法分析
rtc main.rs --lex-only         # 输出 Token 流
rtc main.rs --emit ast         # 打印 AST 结构
rtc main.rs -v                 # 显示各阶段耗时
```

## 完整命令行参数

```
rtc [OPTIONS] <INPUT>

Arguments:
  <INPUT>  源文件路径

Options:
  -o, --output <FILE>     输出文件名
  -c, --compile-only      仅编译到 .o (实际输出 LLVM IR)
  -S, --emit-ir           输出 LLVM IR
  -O, --opt-level <0-3>   优化级别 (暂未实现)
  -v, --verbose           显示编译各阶段信息
  --emit <tokens|ast|ir>  输出中间表示
  -C, --check             仅类型检查
  --parse-only            仅语法分析
  --lex-only              仅词法分析 (输出 Token 流)
  --dump-scope            导出符号表 (暂未实现)
```

## 支持的语言特性

### 基础类型

```rust
fn main() {
    let a: i8 = -128i8;
    let b: i16 = 32767i16;
    let c: i32 = 42;
    let d: i64 = 9223372036854775807i64;
    let e: u8 = 255u8;
    let f: u32 = 4294967295u32;
    let g: u64 = 18446744073709551615u64;
    let h: f32 = 3.14f32;
    let i: f64 = 2.718281828;
    let j: bool = true;
    let k: char = 'A';
    let l: isize = -1isize;
    let m: usize = 100usize;
}
```

### 控制流

```rust
fn main() {
    // if/else
    let x: i32 = if 2 > 1 { 10 } else { 20 };

    // loop + break
    let mut i: i32 = 0;
    loop {
        i = i + 1;
        if i > 5 { break; }
    }

    // while
    let mut j: i32 = 0;
    while j < 10 {
        j = j + 1;
        if j == 5 { continue; }
    }

    // for
    for k in 0..10 {
        let _: i32 = k;
    }

    // return
    fn answer() -> i32 { return 42; }
}
```

### 函数

```rust
fn add(a: i32, b: i32) -> i32 { return a + b; }
fn swap(x: i32, y: i32) -> (i32, i32) { return (y, x); }

fn main() {
    let _s: i32 = add(3, 4);
    let (a, b): (i32, i32) = swap(1, 2);
}
```

### 结构体

```rust
struct Point { x: i32, y: i32 }

fn main() {
    let p: Point = Point { x: 1, y: 2 };
    let _x: i32 = p.x;
}
```

### 元组

```rust
fn main() {
    let t: (i32, f64) = (42, 3.14);
    let _a: i32 = t.0;
    let _b: f64 = t.1;
    let (x, y): (i32, f64) = t;  // 元组解构
}
```

### 枚举 + match

```rust
fn main() {
    let x: i32 = 2;
    let _y: i32 = match x {
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
    // let _x: i32 = a.v;    // ❌ 编译错误: use of moved value

    let x: i32 = 42;
    let y: i32 = x;           // ✅ i32 是 Copy 的
    let z: i32 = x;           // ✅ 可以继续使用

    let r1: &i32 = &y;
    let r2: &i32 = &y;        // ✅ 多个共享借出
    // let r3: &mut i32 = &mut y;  // ❌ 编译错误: 已有共享借出
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
    let _b = None;
}
```

### 模块

```rust
// main.rs
mod math;                      // 自动加载 math.rs

fn main() {
    let _x: i32 = add(3, 4);
}

// math.rs
pub fn add(a: i32, b: i32) -> i32 { return a + b; }
```

## 测试

```bash
bash scripts/test-suite.sh               # 128 测试用例
bash scripts/test-suite.sh --rustc-verify # + rustc 交叉验证
```

## 输出示例

### LLVM IR

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

### 错误诊断

```rust
fn main() { let x: i32 = true; }
```

```
Error: test.rs:1:23: mismatched types: expected i32, found bool
   ╭─[test.rs:1:1]
   │
 1 │ fn main() { let x: i32 = true; }
   │                       ───┬───
   │                          ╰───── mismatched types: expected i32, found bool
───╯
compilation failed with 1 errors
```
