//! The structured types exist to give values that could be expressed with
//! [`Dict`]s and [`List`]s a known structure and meaning that tools / scripts
//! can rely on. The number of these types is kept low to improve reuseability.
//! They are useful to force tools / scripts to decide on one specific structure
//! and to increase compatibility. They also increase maintenance burden, which
//! means that for niche applications it is preferred that tool + script agree
//! on a structure and use dynamic types instead of extending the toolapi.

pub enum Value {
    // Dynamic types - contain more values
    Dict(dynamic::Dict),
    List(dynamic::List),
    // Atomic types - newtypes for consistency
    None(atomic::None),
    Bool(atomic::Bool),
    Int(atomic::Int),
    Float(atomic::Float),
    Complex(atomic::Complex),
    Vec3(atomic::Vec3),
    Vec4(atomic::Vec4),
    Str(atomic::Str),
    // Structured types - fixed lists / dicts with meaning
    InstantSeqEvent(structured::InstantSeqEvent),
    // Tensor types - arrays with singular type
    Tensor1D(tensor::Tensor<1>),
    Tensor2D(tensor::Tensor<2>),
    Tensor3D(tensor::Tensor<3>),
    Tensor4D(tensor::Tensor<4>),
}

pub mod dynamic {
    use super::Value;
    use std::collections::HashMap;

    pub struct Dict(pub HashMap<String, Value>);
    pub struct List(pub Vec<Value>);
}

pub mod atomic {
    use num_complex::Complex64;

    pub struct None;
    pub struct Bool(pub bool);
    pub struct Int(pub i64);
    pub struct Float(pub f64);
    pub struct Complex(pub Complex64);
    pub struct Vec3(pub [f64; 3]);
    pub struct Vec4(pub [f64; 4]);
    pub struct Str(pub String);
}

pub mod tensor {
    use super::atomic;

    /// All atomic and structured types from [`Value`], but as [`Vec`]s.
    /// Improves over a [`List`] of a single type by pulling the type out.
    pub enum ValueVec {
        None(Vec<atomic::None>),
        Bool(Vec<atomic::Bool>),
        Int(Vec<atomic::Int>),
        Float(Vec<atomic::Float>),
        Complex(Vec<atomic::Complex>),
        Vec3(Vec<atomic::Vec3>),
        Vec4(Vec<atomic::Vec4>),
        Str(Vec<atomic::Str>),
    }

    pub struct Tensor<const NDIM: usize> {
        pub size: [u64; NDIM],
        pub data: ValueVec,
    }

    // TODO: introduce a static tensor of fixed type as a helper: We don't want
    // to implement a new type for every tensor type for the API (which is used
    // by python, WASM etc...) but we could do a Rust helper that has static
    // typing and can extract the dynamic `Tensor` variant (fail on wrong type)
    // 
    /// This cannot be found in the `Value` struct and is for Rust usage only
    pub struct StaticTensor<T, const NDIM: usize> {
        pub size: [u64; NDIM],
        pub data: Vec<T>,
    }

    // TODO: implement - we need extraction impl for ValueVec and Value to convert
    // them into the contained data if types match
    impl<const NDIM: usize, T> TryFrom<Tensor<NDIM>> for StaticTensor<T, NDIM> {
        type Error;
    
        fn try_from(value: Tensor<NDIM>) -> Result<Self, Self::Error> {
            todo!()
        }
    }
}

pub mod structured {
    use super::atomic::*;

    pub enum InstantSeqEvent {
        Pulse { angle: Float, phase: Float },
        Fid { kt: Vec4 },
        Adc { phase: Float },
    }

    // TODO: implement MultiTissuePhantom - maybe it is sufficient to implement the Tissue
    // and use a dynamic Dict for multi-tissue?
}
