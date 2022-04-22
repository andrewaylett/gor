use crate::AstError;
use gor_parse::Rule;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum BinOp {
    Eq,
    Neq,
    Lt,
    Leq,
    Gt,
    Geq,
    Add,
    Sub,
    BitOr,
    BitXor,
    Mul,
    Div,
    Modulo,
    Shl,
    Shr,
    BitAnd,
    BitClear,
}

impl TryFrom<Rule> for BinOp {
    type Error = AstError;

    fn try_from(value: Rule) -> std::result::Result<Self, Self::Error> {
        Ok(match value {
            Rule::eq => BinOp::Eq,
            Rule::neq => BinOp::Neq,
            Rule::lt => BinOp::Lt,
            Rule::leq => BinOp::Leq,
            Rule::gt => BinOp::Gt,
            Rule::geq => BinOp::Geq,
            Rule::add => BinOp::Add,
            Rule::sub => BinOp::Sub,
            Rule::bit_or => BinOp::BitOr,
            Rule::bit_xor => BinOp::BitXor,
            Rule::mul => BinOp::Mul,
            Rule::div => BinOp::Div,
            Rule::modulo => BinOp::Modulo,
            Rule::shl => BinOp::Shl,
            Rule::shr => BinOp::Shr,
            Rule::bit_and => BinOp::BitAnd,
            Rule::bit_clear => BinOp::BitClear,
            r => return Err(AstError::InvalidRule("BinOp", r)),
        })
    }
}