use crate::eval::RuntimeError;
use crate::Result as LuaResult;
use crate::Value;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum UniOp {
    Negate,
}

impl UniOp {
    pub(crate) fn static_apply(&self, v: Value) -> core::result::Result<Value, RuntimeError> {
        let v = v.as_int()?;
        let v = match self {
            UniOp::Negate => -v,
        };
        Ok(Value::Int(v))
    }

    pub(crate) fn evaluate(&self, value: Value) -> LuaResult<Value> {
        self.static_apply(value).map_err(Into::into)
    }
}
