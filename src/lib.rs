use anyhow::Result;

use crate::ast::Expression;
use crate::eval::Value;

mod ast;
pub mod parse;

mod eval {
    use anyhow::anyhow;
    use anyhow::Result;

    #[derive(Debug, PartialEq)]
    pub enum Value {
        Int(i64),
        String(String),
    }

    impl Value {
        pub fn as_int(&self) -> Result<i64> {
            if let Value::Int(n) = self {
                Ok(*n)
            } else {
                Err(anyhow!("Not an integer"))
            }
        }
    }
}

pub fn try_static_eval(exp: &Expression) -> Result<Value> {
    Ok(match exp {
        Expression::BinOp { left, op, right } => {
            op.static_apply(try_static_eval(left)?, try_static_eval(right)?)?
        }
        Expression::String(_) => {
            todo!()
        }
        Expression::Number(n) => Value::Int(*n),
        Expression::Name(_) => {
            todo!()
        }
        Expression::UniOp { .. } => {
            todo!()
        }
    })
}

#[cfg(test)]
mod test {
    use anyhow::Result;

    use crate::parse::test::parse_expression;
    use crate::{try_static_eval, Expression, Value};

    #[test]
    fn static_eval_int_addition() -> Result<()> {
        let parse = parse_expression("1+2")?;
        let exp: Expression = parse.try_into()?;
        let result = try_static_eval(&exp)?;
        assert_eq!(result, Value::Int(3));
        Ok(())
    }
}
