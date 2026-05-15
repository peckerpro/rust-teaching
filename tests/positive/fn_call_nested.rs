fn sq(x: i32) -> i32 { return x * x; }
fn sum_sq(a: i32, b: i32) -> i32 { return sq(a) + sq(b); }
fn main() { let x: i32 = sum_sq(3, 4); }
