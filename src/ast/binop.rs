use crate::ast::AstError;
use crate::eval::RuntimeError;
use crate::parse::Rule;
use crate::Result as LuaResult;
use crate::Value;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
    Modulo,
}

impl BinOp {
    pub(crate) fn static_apply(
        &self,
        l: Value,
        r: Value,
    ) -> core::result::Result<Value, RuntimeError> {
        let l = l.as_int()?;
        let r = r.as_int()?;
        let v = match self {
            BinOp::Add => l + r,
            BinOp::Sub => l - r,
            BinOp::Mul => l * r,
            BinOp::Div => l / r,
            BinOp::Pow => l ^ r,
            BinOp::Modulo => l % r,
        };
        Ok(Value::Int(v))
    }

    pub(crate) fn evaluate(&self, left: Value, right: Value) -> LuaResult<Value> {
        self.static_apply(left, right).map_err(Into::into)
    }
}

impl TryFrom<Rule> for BinOp {
    type Error = AstError;

    fn try_from(value: Rule) -> std::result::Result<Self, Self::Error> {
        Ok(match value {
            Rule::add => BinOp::Add,
            Rule::sub => BinOp::Sub,
            Rule::mul => BinOp::Mul,
            Rule::div => BinOp::Div,
            Rule::pow => BinOp::Pow,
            Rule::modulo => BinOp::Modulo,
            r => return Err(AstError::InvalidRule("BinOp", r)),
        })
    }
}
