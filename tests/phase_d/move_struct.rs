struct Val { v: i32 }

fn main() {
    let a: Val = Val { v: 1 };
    let _b: Val = a;
}
