//! This module implements the .get() function, which enables to extract static
//! types from dynamically typed Values.

// TODO: proper error handling (instead of options)

use std::collections::HashMap;
use std::any::{type_name, type_name_of_val};

use num_complex::Complex64;

use crate::{TypeExtractionError, value::typed::{TypedDict, TypedList}};

use super::Value;

impl Value {
    /// TODO: implement recursive indexing
    pub fn index(&self, index: impl Into<Index>) -> Option<Value> {
        let index = index.into();

        // typed list and dict elements will be converted into dynamic values.
        // for efficient use, extract the whole list / dict with typing instead
        use crate::value::typed::{TypedDict, TypedList};
        match index {
            Index::Array(index) => match self {
                Value::List(list) => list.0.get(index).cloned(),
                Value::TypedList(list) => match list {
                    TypedList::None(items) => items.get(index).cloned().map(Value::None),
                    TypedList::Bool(items) => items.get(index).cloned().map(Value::Bool),
                    TypedList::Int(items) => items.get(index).cloned().map(Value::Int),
                    TypedList::Float(items) => items.get(index).cloned().map(Value::Float),
                    TypedList::Str(items) => items.get(index).cloned().map(Value::Str),
                    TypedList::Complex(items) => items.get(index).cloned().map(Value::Complex),
                    TypedList::Vec3(items) => items.get(index).cloned().map(Value::Vec3),
                    TypedList::Vec4(items) => items.get(index).cloned().map(Value::Vec4),
                    TypedList::InstantSeqEvent(items) => {
                        items.get(index).cloned().map(Value::InstantSeqEvent)
                    }
                    TypedList::Volume(items) => items.get(index).cloned().map(Value::Volume),
                    TypedList::SegmentedPhantom(items) => {
                        items.get(index).cloned().map(Value::SegmentedPhantom)
                    }
                    TypedList::PhantomTissue(items) => {
                        items.get(index).cloned().map(Value::PhantomTissue)
                    }
                },
                _ => None,
            },
            Index::Dict(k) => match self {
                Value::Dict(dict) => dict.0.get(&k).cloned(),
                Value::TypedDict(dict) => match dict {
                    TypedDict::None(items) => items.get(&k).cloned().map(Value::None),
                    TypedDict::Bool(items) => items.get(&k).cloned().map(Value::Bool),
                    TypedDict::Int(items) => items.get(&k).cloned().map(Value::Int),
                    TypedDict::Float(items) => items.get(&k).cloned().map(Value::Float),
                    TypedDict::Str(items) => items.get(&k).cloned().map(Value::Str),
                    TypedDict::Complex(items) => items.get(&k).cloned().map(Value::Complex),
                    TypedDict::Vec3(items) => items.get(&k).cloned().map(Value::Vec3),
                    TypedDict::Vec4(items) => items.get(&k).cloned().map(Value::Vec4),
                    TypedDict::InstantSeqEvent(items) => {
                        items.get(&k).cloned().map(Value::InstantSeqEvent)
                    }
                    TypedDict::Volume(items) => items.get(&k).cloned().map(Value::Volume),
                    TypedDict::SegmentedPhantom(items) => {
                        items.get(&k).cloned().map(Value::SegmentedPhantom)
                    }
                    TypedDict::PhantomTissue(items) => {
                        items.get(&k).cloned().map(Value::PhantomTissue)
                    }
                },
                _ => None,
            },
        }
    }
}

// TODO: allow nested paths like "tissues.0.t2".
// This would need a reworked index enum and a fancy From<String> impl
pub enum Index {
    Array(usize),
    Dict(String),
}

impl From<usize> for Index {
    fn from(value: usize) -> Self {
        Self::Array(value)
    }
}

impl From<&str> for Index {
    fn from(value: &str) -> Self {
        Self::Dict(value.to_owned())
    }
}

impl From<String> for Index {
    fn from(value: String) -> Self {
        Self::Dict(value)
    }
}

// TODO: use macro for this
macro_rules! impl_conversion {
    ($typ:ty, $variant:ident) => {
        impl From<$typ> for Value {
            fn from(value: $typ) -> Self {
                Self::$variant(value)
            }
        }
        impl TryFrom<Value> for $typ {
            type Error = TypeExtractionError;

            fn try_from(value: Value) -> Result<Self, Self::Error> {
                match value {
                    Value::$variant(value) => Ok(value),
                    _ => Err(TypeExtractionError {
                        from: type_name_of_val(&value),
                        into: type_name::<$typ>(),
                    }),
                }
            }
        }
        impl TryFrom<TypedList> for Vec<$typ> {
            type Error = TypeExtractionError;

            fn try_from(value: TypedList) -> Result<Self, Self::Error> {
                match value {
                    TypedList::$variant(value) => Ok(value),
                    _ => Err(TypeExtractionError {
                        from: type_name_of_val(&value),
                        into: type_name::<Vec<$typ>>(),
                    }),
                }
            }
        }
        impl TryFrom<TypedDict> for HashMap<String, $typ> {
            type Error = TypeExtractionError;

            fn try_from(value: TypedDict) -> Result<Self, Self::Error> {
                match value {
                    TypedDict::$variant(value) => Ok(value),
                    _ => Err(TypeExtractionError {
                        from: type_name_of_val(&value),
                        into: type_name::<Vec<$typ>>(),
                    }),
                }
            }
        }
    };
}

use super::{atomic, structured};
impl_conversion!((), None);
impl_conversion!(bool, Bool);
impl_conversion!(i64, Int);
impl_conversion!(f64, Float);
impl_conversion!(String, Str);
impl_conversion!(Complex64, Complex);
impl_conversion!(atomic::Vec3, Vec3);
impl_conversion!(atomic::Vec4, Vec4);
impl_conversion!(structured::InstantSeqEvent, InstantSeqEvent);
impl_conversion!(structured::Volume, Volume);
impl_conversion!(structured::SegmentedPhantom, SegmentedPhantom);
impl_conversion!(structured::PhantomTissue, PhantomTissue);

// TODO: we are missing conversions for (Typed)List/Dict. The above already
// implements some of them, wait for an example to show exact traits missing!