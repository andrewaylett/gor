use anyhow::Result;

use crate::ast::Expression;
use crate::eval::Value;

pub mod parse;

mod ast {
    use pest::iterators::Pairs;

    #[derive(Debug)]
    pub struct Expression {}

    impl From<Pairs<'_, crate::parse::Rule>> for Expression {
        fn from(_: Pairs<crate::parse::Rule>) -> Self {
            unimplemented!()
        }
    }
}

mod eval {
    #[derive(Debug, PartialEq)]
    pub enum Value {
        Int(i64),
    }
}

pub fn try_static_eval(exp: &Expression) -> Result<Value> {
    unimplemented!("{:?}", exp)
}

#[cfg(test)]
mod test {
    use crate::parse::test::parse_expression;
    use crate::{try_static_eval, Expression, Value};

    #[test]
    #[should_panic(expected = "not implemented")]
    fn static_eval_int_addition() {
        let parse = parse_expression("1+2").unwrap();
        let exp: Expression = parse.into();
        let result = try_static_eval(&exp).unwrap();
        assert_eq!(result, Value::Int(3));
    }
}
