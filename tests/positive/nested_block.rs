fn main() {
    let x: i32 = 1;
    {
        let y: i32 = 2;
        let z: i32 = x + y;
    }
    let w: i32 = x;
}
