use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::error::{ConversionError, LookupError};

#[derive(Debug, Serialize, Deserialize)]
pub struct ValueDict(pub HashMap<String, Value>);

impl ValueDict {
    pub fn pop<T>(&mut self, key: &str) -> Result<T, LookupError>
    where
        T: TryFrom<Value, Error = ConversionError>,
    {
        match self.0.remove(key) {
            Some(value) => Ok(value.try_into().map_err(LookupError::ConversionError)?),
            None => Err(LookupError::KeyError),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Value {
    None(()),
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
}

// =====================================================
// CONVERSION: Dynamic typed Value <-> Static typed Rust
// =====================================================

impl Value {
    fn type_name(&self) -> &'static str {
        use std::any::type_name_of_val;

        match self {
            Value::None(x) => type_name_of_val(x),
            Value::Bool(x) => type_name_of_val(x),
            Value::Int(x) => type_name_of_val(x),
            Value::Float(x) => type_name_of_val(x),
            Value::String(x) => type_name_of_val(x),
        }
    }
}

macro_rules! impl_value {
    ($rust_type:ty, $value_type:ident) => {
        impl From<$rust_type> for Value {
            fn from(value: $rust_type) -> Self {
                Value::$value_type(value)
            }
        }

        impl TryFrom<Value> for $rust_type {
            type Error = ConversionError;

            fn try_from(value: Value) -> Result<Self, Self::Error> {
                match value {
                    Value::$value_type(value) => Ok(value),
                    _ => Err(ConversionError {
                        from: value.type_name(),
                        into: std::any::type_name::<$rust_type>(),
                    }),
                }
            }
        }
    };
}

impl_value!((), None);
impl_value!(bool, Bool);
impl_value!(i64, Int);
impl_value!(f64, Float);
impl_value!(String, String);
