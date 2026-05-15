fn main() {
    let x: i32 = 2;
    let _y: i32 = match x {
        1 => 10,
        2 => 20,
        _ => 30,
    };
}
