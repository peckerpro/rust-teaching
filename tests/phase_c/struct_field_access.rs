struct Point { x: i32, y: i32 }
fn dist_sq(p: Point) -> i32 { return p.x * p.x + p.y * p.y; }
fn main() {
    let pt: Point = Point { x: 3, y: 4 };
    let _d: i32 = dist_sq(pt);
}
