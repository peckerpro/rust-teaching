struct Val { v: i32 }

fn consume(x: Val) -> i32 { return x.v; }

fn main() {
    let v: Val = Val { v: 10 };
    let _r: i32 = consume(v);
}
