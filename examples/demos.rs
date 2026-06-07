use std::io::{Read, stdin};

use calculator::Parser;
fn main() {
    let mut buf = String::new();
    stdin().read_to_string(&mut buf).expect("missing expr");
    let mut parser = Parser::new(&buf);
    let ast = parser.parse().expect(&format!("invalid expr {}", buf));
    println!("{}", ast.value());
}
