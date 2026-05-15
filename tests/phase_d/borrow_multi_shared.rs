fn main() {
    let x: i32 = 10;
    let r1: &i32 = &x;
    let r2: &i32 = &x;
    let _a: i32 = *r1;
    let _b: i32 = *r2;
}
