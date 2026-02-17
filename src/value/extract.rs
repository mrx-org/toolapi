//! This module implements the .get() function, which enables to extract static
//! types from dynamically typed Values.

use std::any::{type_name, type_name_of_val};
use std::collections::HashMap;

use num_complex::Complex64;

use crate::{
    ExtractionError,
    value::typed::{TypedDict, TypedList},
};

use super::Value;

impl Value {
    pub fn get(&self, ptr: impl Into<Pointer>) -> Result<Value, ExtractionError> {
        self._get(&ptr.into().0)
    }

    fn _get(&self, ptr: &[Index]) -> Result<Value, ExtractionError> {
        let index = ptr.first();
        let rest = ptr.get(1..);

        use ExtractionError::*;
        match (self, index, rest) {
            // no indexing: return Value even if it could have contained more nesting
            (value, None, None) => Ok(value.clone()),

            // simple indexing into List / Dict - call recurively into them
            (Value::List(list), Some(Index::Idx(idx)), rest) => get_list(list, idx, rest),
            (Value::Dict(dict), Some(Index::Key(key)), rest) => get_dict(dict, key, rest),
            // typed List / Dict: contain atomic types, must be end of path
            (Value::TypedList(list), Some(Index::Idx(idx)), None) => get_typed_list(list, idx),
            (Value::TypedDict(dict), Some(Index::Key(key)), None) => get_typed_dict(dict, key),
            (Value::TypedList(_), Some(Index::Idx(_)), Some(_)) => Err(TooMuchNesting),
            (Value::TypedDict(_), Some(Index::Key(_)), Some(_)) => Err(TooMuchNesting),

            // Wrong type of index for List / Dict
            (Value::List(_), Some(Index::Key(_)), _) => Err(KeyForList),
            (Value::Dict(_), Some(Index::Idx(_)), _) => Err(IndexForDict),
            (Value::TypedList(_), Some(Index::Key(_)), _) => Err(KeyForList),
            (Value::TypedDict(_), Some(Index::Idx(_)), _) => Err(IndexForDict),

            // Trying to index into a non-list/dict value
            (_, Some(_), _) => Err(TooMuchNesting),

            // ptr.get(0) = None && ptr.get(1..) = Some: impossible
            (_, None, Some(_)) => unreachable!(),
        }
    }
}

fn get_list(
    list: &super::dynamic::List,
    index: &usize,
    rest: Option<&[Index]>,
) -> Result<Value, ExtractionError> {
    list.0
        .get(*index)
        .ok_or(ExtractionError::IndexOutOfBounds)
        .and_then(|value| value._get(rest.unwrap_or_default()))
}

fn get_dict(
    dict: &super::dynamic::Dict,
    key: &str,
    rest: Option<&[Index]>,
) -> Result<Value, ExtractionError> {
    dict.0
        .get(key)
        .ok_or(ExtractionError::KeyNotFound)
        .and_then(|value| value._get(rest.unwrap_or_default()))
}

fn get_typed_list(list: &TypedList, idx: &usize) -> Result<Value, ExtractionError> {
    match list {
        TypedList::None(items) => items.get(*idx).cloned().map(Value::None),
        TypedList::Bool(items) => items.get(*idx).cloned().map(Value::Bool),
        TypedList::Int(items) => items.get(*idx).cloned().map(Value::Int),
        TypedList::Float(items) => items.get(*idx).cloned().map(Value::Float),
        TypedList::Str(items) => items.get(*idx).cloned().map(Value::Str),
        TypedList::Complex(items) => items.get(*idx).cloned().map(Value::Complex),
        TypedList::Vec3(items) => items.get(*idx).cloned().map(Value::Vec3),
        TypedList::Vec4(items) => items.get(*idx).cloned().map(Value::Vec4),
        TypedList::InstantSeqEvent(items) => items.get(*idx).cloned().map(Value::InstantSeqEvent),
        TypedList::Volume(items) => items.get(*idx).cloned().map(Value::Volume),
        TypedList::SegmentedPhantom(items) => items.get(*idx).cloned().map(Value::SegmentedPhantom),
        TypedList::PhantomTissue(items) => items.get(*idx).cloned().map(Value::PhantomTissue),
    }
    .ok_or(ExtractionError::IndexOutOfBounds)
}

fn get_typed_dict(dict: &TypedDict, key: &str) -> Result<Value, ExtractionError> {
    match dict {
        TypedDict::None(items) => items.get(key).cloned().map(Value::None),
        TypedDict::Bool(items) => items.get(key).cloned().map(Value::Bool),
        TypedDict::Int(items) => items.get(key).cloned().map(Value::Int),
        TypedDict::Float(items) => items.get(key).cloned().map(Value::Float),
        TypedDict::Str(items) => items.get(key).cloned().map(Value::Str),
        TypedDict::Complex(items) => items.get(key).cloned().map(Value::Complex),
        TypedDict::Vec3(items) => items.get(key).cloned().map(Value::Vec3),
        TypedDict::Vec4(items) => items.get(key).cloned().map(Value::Vec4),
        TypedDict::InstantSeqEvent(items) => items.get(key).cloned().map(Value::InstantSeqEvent),
        TypedDict::Volume(items) => items.get(key).cloned().map(Value::Volume),
        TypedDict::SegmentedPhantom(items) => items.get(key).cloned().map(Value::SegmentedPhantom),
        TypedDict::PhantomTissue(items) => items.get(key).cloned().map(Value::PhantomTissue),
    }
    .ok_or(ExtractionError::KeyNotFound)
}

/// Use with [`Value::index`] to extract from a nested [`Dict`] / [`List`].
///
/// A [`Pointer`] is a '/' separated path, containing
/// - strings to index into a [`Dict`]
/// - numbers to index into a [`List`]
///
/// Note that [`Dict`] keys can be numbers, empty strings, ... as well.
///
/// # Examples
/// ```ignore
/// "tissues/3/density" // Extract from a nested path
/// "2/some_property" // Top level is an array
/// "" // returns whole `Value` unchanged
/// "empty//key" // Empty key in `Dict` at second level
/// ```
pub struct Pointer(Vec<Index>);

enum Index {
    Key(String),
    Idx(usize),
}

impl From<usize> for Pointer {
    fn from(value: usize) -> Self {
        Self(vec![Index::Idx(value)])
    }
}

impl From<&str> for Pointer {
    fn from(value: &str) -> Self {
        Self(
            value
                .split('/')
                .map(|element| match element.parse::<usize>() {
                    Ok(index) => Index::Idx(index),
                    Err(_) => Index::Key(element.to_string()),
                })
                .collect(),
        )
    }
}

impl From<String> for Pointer {
    fn from(value: String) -> Self {
        Self::from(value.as_str())
    }
}

macro_rules! impl_conversion {
    ($typ:ty, $variant:ident) => {
        // ============================
        // Rust -> Value
        // ============================
        impl From<$typ> for Value {
            fn from(value: $typ) -> Self {
                Self::$variant(value)
            }
        }
        impl From<Vec<$typ>> for Value {
            fn from(value: Vec<$typ>) -> Self {
                Self::TypedList(TypedList::$variant(value))
            }
        }
        impl From<HashMap<String, $typ>> for Value {
            fn from(value: HashMap<String, $typ>) -> Self {
                Self::TypedDict(TypedDict::$variant(value))
            }
        }

        // ============================
        // Value -> Rust
        // ============================
        impl TryFrom<Value> for $typ {
            type Error = ExtractionError;

            fn try_from(value: Value) -> Result<Self, Self::Error> {
                match value {
                    Value::$variant(value) => Ok(value),
                    _ => Err(ExtractionError::TypeMismatch {
                        from: type_name_of_val(&value).to_string(),
                        into: type_name::<$typ>().to_string(),
                    }),
                }
            }
        }

        // ============================
        // TypedList -> Vec
        // ============================
        impl TryFrom<TypedList> for Vec<$typ> {
            type Error = ExtractionError;

            fn try_from(value: TypedList) -> Result<Self, Self::Error> {
                match value {
                    TypedList::$variant(value) => Ok(value),
                    _ => Err(ExtractionError::TypeMismatch {
                        from: type_name_of_val(&value).to_string(),
                        into: type_name::<Vec<$typ>>().to_string(),
                    }),
                }
            }
        }
        impl TryFrom<Value> for Vec<$typ> {
            type Error = ExtractionError;

            fn try_from(value: Value) -> Result<Self, Self::Error> {
                match value {
                    Value::TypedList(TypedList::$variant(value)) => Ok(value),
                    _ => Err(ExtractionError::TypeMismatch {
                        from: type_name_of_val(&value).to_string(),
                        into: type_name::<Vec<$typ>>().to_string(),
                    }),
                }
            }
        }

        // ============================
        // TypedDict -> HashMap
        // ============================
        impl TryFrom<TypedDict> for HashMap<String, $typ> {
            type Error = ExtractionError;

            fn try_from(value: TypedDict) -> Result<Self, Self::Error> {
                match value {
                    TypedDict::$variant(value) => Ok(value),
                    _ => Err(ExtractionError::TypeMismatch {
                        from: type_name_of_val(&value).to_string(),
                        into: type_name::<HashMap<String, $typ>>().to_string(),
                    }),
                }
            }
        }
        impl TryFrom<Value> for HashMap<String, $typ> {
            type Error = ExtractionError;

            fn try_from(value: Value) -> Result<Self, Self::Error> {
                match value {
                    Value::TypedDict(TypedDict::$variant(value)) => Ok(value),
                    _ => Err(ExtractionError::TypeMismatch {
                        from: type_name_of_val(&value).to_string(),
                        into: type_name::<HashMap<String, $typ>>().to_string(),
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
