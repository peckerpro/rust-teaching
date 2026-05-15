fn id<T>(x: T) -> T { return x; }
fn main() {
    let a: i32 = id(10);
    let b: f64 = id(2.5);
    let _c: i32 = a;
    let _d: f64 = b;
}
