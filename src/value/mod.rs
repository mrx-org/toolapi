//! The structured types exist to give values that could be expressed with
//! [`Dict`]s and [`List`]s a known structure and meaning that tools / scripts
//! can rely on. The number of these types is kept low to improve reuseability.
//! They are useful to force tools / scripts to decide on one specific structure
//! and to increase compatibility. They also increase maintenance burden, which
//! means that for niche applications it is preferred that tool + script agree
//! on a structure and use dynamic types instead of extending the toolapi.

use num_complex::Complex64;
use serde::{Deserialize, Serialize};

mod extract;
mod utils;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Value {
    // Atomic types - think of py and wasm compatibility (e.g. single int type)
    None(()),
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
    Complex(Complex64),
    Vec3(atomic::Vec3),
    Vec4(atomic::Vec4),
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
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Vec3(pub [f64; 3]);
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Vec4(pub [f64; 4]);
}

pub mod structured {
    use super::atomic::*;
    use super::typed::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum InstantSeqEvent {
        Pulse { angle: f64, phase: f64 },
        Fid { kt: Vec4 },
        Adc { phase: f64 },
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

        pub t1: f64,
        pub t2: f64,
        pub t2dash: f64,
        pub adc: f64,
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
    use num_complex::Complex64;
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;

    // These types do not contain Lists / Dicts. They are meant for
    // efficiently packing values of a single type and do not support
    // nested indexing (see extract.rs). All other Value types are supported.

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum TypedList {
        None(Vec<()>),
        Bool(Vec<bool>),
        Int(Vec<i64>),
        Float(Vec<f64>),
        Str(Vec<String>),
        Complex(Vec<Complex64>),
        Vec3(Vec<atomic::Vec3>),
        Vec4(Vec<atomic::Vec4>),
        InstantSeqEvent(Vec<structured::InstantSeqEvent>),
        Volume(Vec<structured::Volume>),
        SegmentedPhantom(Vec<structured::SegmentedPhantom>),
        PhantomTissue(Vec<structured::PhantomTissue>),
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum TypedDict {
        None(HashMap<String, ()>),
        Bool(HashMap<String, bool>),
        Int(HashMap<String, i64>),
        Float(HashMap<String, f64>),
        Str(HashMap<String, String>),
        Complex(HashMap<String, Complex64>),
        Vec3(HashMap<String, atomic::Vec3>),
        Vec4(HashMap<String, atomic::Vec4>),
        InstantSeqEvent(HashMap<String, structured::InstantSeqEvent>),
        Volume(HashMap<String, structured::Volume>),
        SegmentedPhantom(HashMap<String, structured::SegmentedPhantom>),
        PhantomTissue(HashMap<String, structured::PhantomTissue>),
    }
}
