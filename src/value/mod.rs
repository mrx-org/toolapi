use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::error::{LookupError, TypeExtractionError};

mod misc;
mod phantom;
mod sequence;
pub use misc::*;
pub use phantom::*;
pub use sequence::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Value {
    None(()),
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Signal(Signal),
    Encoding(Encoding),
    // TODO: reduce / unify
    TissueProperties(TissueProperties),
    MultiTissuePhantom(MultiTissuePhantom),
    VoxelPhantom(VoxelPhantom),
    VoxelGridPhantom(VoxelGridPhantom),
    // TODO: rethink naming (especially of the structs inside of the seqs!)
    EventSeq(EventSeq),
    BlockSeq(BlockSeq),
}

// =============================================================
// ValueDict: Collection of values used as Tool input and output
// =============================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct ValueDict(HashMap<String, Value>);

impl<const N: usize> From<[(String, Value); N]> for ValueDict {
    fn from(value: [(String, Value); N]) -> Self {
        Self(HashMap::from(value))
    }
}

impl<const N: usize> From<[(&str, Value); N]> for ValueDict {
    fn from(value: [(&str, Value); N]) -> Self {
        Self(HashMap::from_iter(
            value.into_iter().map(|(k, v)| (k.to_owned(), v.into())),
        ))
    }
}

impl FromIterator<(String, Value)> for ValueDict
{
    fn from_iter<T: IntoIterator<Item = (String, Value)>>(iter: T) -> Self {
        Self(HashMap::from_iter(iter))
    }
}

impl IntoIterator for ValueDict {
    type Item = (String, Value);
    type IntoIter = std::collections::hash_map::IntoIter<String, Value>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl ValueDict {
    pub fn pop<T>(&mut self, key: &str) -> Result<T, LookupError>
    where
        T: TryFrom<Value, Error = TypeExtractionError>,
    {
        match self.0.remove(key) {
            Some(value) => Ok(value.try_into()?),
            None => Err(LookupError::KeyError(key.to_owned())),
        }
    }
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
            Value::Signal(x) => type_name_of_val(x),
            Value::Encoding(x) => type_name_of_val(x),
            Value::TissueProperties(x) => type_name_of_val(x),
            Value::MultiTissuePhantom(x) => type_name_of_val(x),
            Value::VoxelPhantom(x) => type_name_of_val(x),
            Value::VoxelGridPhantom(x) => type_name_of_val(x),
            Value::EventSeq(x) => type_name_of_val(x),
            Value::BlockSeq(x) => type_name_of_val(x),
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
            type Error = TypeExtractionError;

            fn try_from(value: Value) -> Result<Self, Self::Error> {
                match value {
                    Value::$value_type(value) => Ok(value),
                    _ => Err(TypeExtractionError {
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
impl_value!(Signal, Signal);
impl_value!(Encoding, Encoding);
impl_value!(TissueProperties, TissueProperties);
impl_value!(MultiTissuePhantom, MultiTissuePhantom);
impl_value!(VoxelPhantom, VoxelPhantom);
impl_value!(VoxelGridPhantom, VoxelGridPhantom);
impl_value!(EventSeq, EventSeq);
impl_value!(BlockSeq, BlockSeq);
