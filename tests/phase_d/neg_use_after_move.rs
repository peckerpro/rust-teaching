struct Point { x: i32, y: i32 }

fn main() {
    let p: Point = Point { x: 1, y: 2 };
    let _q: Point = p;
    let _x: i32 = p.x;
}
