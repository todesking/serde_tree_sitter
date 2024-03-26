#[derive(serde::Deserialize)]
struct Document {}

#[derive(serde::Deserialize)]
enum Value {
    Objec(),
    Array(Vec<Value>),
    Number(),
    String(),
    True,
    False,
    Null,
}

fn main() {
    println!("json");
}
