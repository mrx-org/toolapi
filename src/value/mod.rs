//! The structured types exist to give values that could be expressed with
//! [`Dict`]s and [`List`]s a known structure and meaning that tools / scripts
//! can rely on. The number of these types is kept low to improve reuseability.
//! They are useful to force tools / scripts to decide on one specific structure
//! and to increase compatibility. They also increase maintenance burden, which
//! means that for niche applications it is preferred that tool + script agree
//! on a structure and use dynamic types instead of extending the toolapi.

use serde::{Deserialize, Serialize};

mod extract;
mod utils;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Value {
    // Atomic types - newtypes for consistency
    None(atomic::None),
    Bool(atomic::Bool),
    Int(atomic::Int),
    Float(atomic::Float),
    Complex(atomic::Complex),
    Vec3(atomic::Vec3),
    Vec4(atomic::Vec4),
    Str(atomic::Str),
    // Structured types - (MRI) types with semantic meaning
    InstantSeqEvent(structured::InstantSeqEvent),
    Volume(structured::Volume),
    SegmentedPhantom(structured::SegmentedPhantom),
    PhantomTissue(structured::PhantomTissue),
    // Dynamic collections - each value can have a different type
    Dict(dynamic::Dict),
    List(dynamic::List),
    // Static collections - all values have the same type
    TypedDict(typed::TypedDict),
    TypedList(typed::TypedList),
}

pub mod atomic {
    use num_complex::Complex64;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct None;
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Bool(pub bool);
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Int(pub i64);
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Float(pub f64);
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Complex(pub Complex64);
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Vec3(pub [f64; 3]);
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Vec4(pub [f64; 4]);
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Str(pub String);
}

pub mod structured {
    use super::atomic::*;
    use super::typed::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum InstantSeqEvent {
        Pulse { angle: Float, phase: Float },
        Fid { kt: Vec4 },
        Adc { phase: Float },
    }

    /// 3D voxel volume (with affine) of arbitrary (but singular) type
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Volume {
        pub shape: [u64; 3],
        pub affine: [[f64; 4]; 3],
        pub data: TypedList,
    }

    /// This does not follow the NIfTI standard exactly because that allows to
    /// maps for T1, T2 (so that it can describe classical voxel phantoms as well).
    /// Here we want to specifically cater to segmented simulations, so we are
    /// more restrictive. Therefore NIfTI -> [`SegmentedPhantom`] can be lossy.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct SegmentedPhantom {
        pub tissues: Vec<PhantomTissue>,
        pub b1_tx: Vec<Volume>,
        pub b1_rx: Vec<Volume>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct PhantomTissue {
        pub density: Volume,
        pub db0: Volume,

        pub t1: Float,
        pub t2: Float,
        pub t2dash: Float,
        pub adc: Float,
    }
}

pub mod dynamic {
    use super::Value;
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Dict(pub HashMap<String, Value>);
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct List(pub Vec<Value>);
}

/// Contains [`List`]s and [`Dict`]s where all values have the same type
pub mod typed {
    use super::atomic;
    use super::structured;
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum TypedList {
        None(Vec<atomic::None>),
        Bool(Vec<atomic::Bool>),
        Int(Vec<atomic::Int>),
        Float(Vec<atomic::Float>),
        Complex(Vec<atomic::Complex>),
        Vec3(Vec<atomic::Vec3>),
        Vec4(Vec<atomic::Vec4>),
        Str(Vec<atomic::Str>),
        InstantSeqEvent(Vec<structured::InstantSeqEvent>),
        Volume(Vec<structured::Volume>),
        SegmentedPhantom(Vec<structured::SegmentedPhantom>),
        PhantomTissue(Vec<structured::PhantomTissue>),
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum TypedDict {
        None(HashMap<String, atomic::None>),
        Bool(HashMap<String, atomic::Bool>),
        Int(HashMap<String, atomic::Int>),
        Float(HashMap<String, atomic::Float>),
        Complex(HashMap<String, atomic::Complex>),
        Vec3(HashMap<String, atomic::Vec3>),
        Vec4(HashMap<String, atomic::Vec4>),
        Str(HashMap<String, atomic::Str>),
        InstantSeqEvent(HashMap<String, structured::InstantSeqEvent>),
        Volume(HashMap<String, structured::Volume>),
        SegmentedPhantom(HashMap<String, structured::SegmentedPhantom>),
        PhantomTissue(HashMap<String, structured::PhantomTissue>),
    }
}
