fn main() {
    let x: i32 = 10;
    let r: &mut i32 = &mut x;
    let _a: i32 = *r;
    let _b: i32 = x;
}
