fn main() {
    let x: i32 = 0;
    let _y: i32 = match x {
        0 => 1,
        _ => 0,
    };
}
