
use lalrpop_util::lalrpop_mod;

lalrpop_mod!(parser);

mod syntax;

fn main() {
    println!("Hello, world!");

    let p = parser::ExprParser::new();
    let e = p.parse("(3 + x) * 5");
}
