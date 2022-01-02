use lua::parse;
use lua::parse::Rule;

fn main() {
    let parse = parse::parse(Rule::expression, "30+28");
    println!("{:?}", parse)
}
