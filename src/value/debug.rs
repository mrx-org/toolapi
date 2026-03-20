use std::fmt::Debug;

use crate::value::{
    Value, dynamic::{Dict, List}, typed::{TypedDict, TypedList}
};

impl Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None(()) => f.write_str("None"),
            Self::Bool(x) => x.fmt(f),
            Self::Int(x) => write!(f, "{x}i64"),
            Self::Float(x) => write!(f, "{x}f64"),
            Self::Str(x) => x.fmt(f),
            Self::Bytes(x) => write!(f, "<{} bytes>", x.len()),
            Self::Complex(x) => write!(f, "({} + {}i)", x.re, x.im),
            Self::Vec3(x) => write!(f, "v3{:?}", x.0),
            Self::Vec4(x) => write!(f, "v4{:?}", x.0),
            Self::InstantSeqEvent(x) => x.fmt(f),
            Self::Volume(x) => x.fmt(f),
            Self::SegmentedPhantom(x) => x.fmt(f),
            Self::PhantomTissue(x) => x.fmt(f),
            Self::Dict(x) => x.fmt(f),
            Self::List(x) => x.fmt(f),
            Self::TypedDict(x) => x.fmt(f),
            Self::TypedList(x) => x.fmt(f),
        }
    }
}

impl Debug for List {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let len = self.0.len();
        if len <= 10 {
            f.debug_list().entries(&self.0).finish()
        } else {
            let mut list = f.debug_list();
            list.entries(&self.0[..8]);
            list.entry(&Ellipsis(len - 10));
            list.entries(&self.0[len - 2..]);
            list.finish()
        }
    }
}

struct Ellipsis(usize);

impl Debug for Ellipsis {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "... ({} more)", self.0)
    }
}

impl Debug for Dict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // its evident from the "{ ... }" that this is a dict
        self.0.fmt(f)
    }
}

impl Debug for TypedList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None(x) => f.debug_tuple("None").field(x).finish(),
            Self::Bool(x) => f.debug_tuple("Bool").field(x).finish(),
            Self::Int(x) => f.debug_tuple("Int").field(x).finish(),
            Self::Float(x) => f.debug_tuple("Float").field(x).finish(),
            Self::Str(x) => f.debug_tuple("Str").field(x).finish(),
            Self::Bytes(x) => f.debug_tuple("Bytes").field(x).finish(),
            Self::Complex(x) => f.debug_tuple("Complex").field(x).finish(),
            Self::Vec3(x) => f.debug_tuple("Vec3").field(x).finish(),
            Self::Vec4(x) => f.debug_tuple("Vec4").field(x).finish(),
            Self::InstantSeqEvent(x) => f.debug_tuple("InstantSeqEvent").field(x).finish(),
            Self::Volume(x) => f.debug_tuple("Volume").field(x).finish(),
            Self::SegmentedPhantom(x) => f.debug_tuple("SegmentedPhantom").field(x).finish(),
            Self::PhantomTissue(x) => f.debug_tuple("PhantomTissue").field(x).finish(),
        }
    }
}

impl Debug for TypedDict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None(x) => f.debug_tuple("None").field(x).finish(),
            Self::Bool(x) => f.debug_tuple("Bool").field(x).finish(),
            Self::Int(x) => f.debug_tuple("Int").field(x).finish(),
            Self::Float(x) => f.debug_tuple("Float").field(x).finish(),
            Self::Str(x) => f.debug_tuple("Str").field(x).finish(),
            Self::Bytes(x) => f.debug_tuple("Bytes").field(x).finish(),
            Self::Complex(x) => f.debug_tuple("Complex").field(x).finish(),
            Self::Vec3(x) => f.debug_tuple("Vec3").field(x).finish(),
            Self::Vec4(x) => f.debug_tuple("Vec4").field(x).finish(),
            Self::InstantSeqEvent(x) => f.debug_tuple("InstantSeqEvent").field(x).finish(),
            Self::Volume(x) => f.debug_tuple("Volume").field(x).finish(),
            Self::SegmentedPhantom(x) => f.debug_tuple("SegmentedPhantom").field(x).finish(),
            Self::PhantomTissue(x) => f.debug_tuple("PhantomTissue").field(x).finish(),
        }
    }
}
