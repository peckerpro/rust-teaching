fn main() {
    let x: i32 = 10;
    let f = |y: i32| x + y;
    let _r: i32 = f(5);
}
