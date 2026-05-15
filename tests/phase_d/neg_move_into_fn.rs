fn take_point(p: Point) -> i32 { return p.x; }

struct Point { x: i32, y: i32 }

fn main() {
    let pt: Point = Point { x: 1, y: 2 };
    let _x: i32 = take_point(pt);
    let _y: i32 = pt.x;
}
