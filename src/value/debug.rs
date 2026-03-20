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

impl Debug for Dict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt_typed_map(&self.0, "", f)
    }
}

impl Debug for TypedList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None(x) => fmt_typed_list(x, "", f),
            Self::Bool(x) => fmt_typed_list(x, "", f),
            Self::Int(x) => fmt_typed_list(x, "i64", f),
            Self::Float(x) => fmt_typed_list(x, "f64", f),
            Self::Str(x) => fmt_typed_list(x, "", f),
            Self::Bytes(x) => fmt_typed_list(x, "bytes", f),
            Self::Complex(x) => fmt_typed_list(x, "complex", f),
            Self::Vec3(x) => fmt_typed_list(x, "v3", f),
            Self::Vec4(x) => fmt_typed_list(x, "v4", f),
            Self::InstantSeqEvent(x) => fmt_typed_list(x, "", f),
            Self::Volume(x) => fmt_typed_list(x, "", f),
            Self::SegmentedPhantom(x) => fmt_typed_list(x, "", f),
            Self::PhantomTissue(x) => fmt_typed_list(x, "", f),
        }
    }
}

impl Debug for TypedDict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None(x) => fmt_typed_map(x, "", f),
            Self::Bool(x) => fmt_typed_map(x, "", f),
            Self::Int(x) => fmt_typed_map(x, "i64", f),
            Self::Float(x) => fmt_typed_map(x, "f64", f),
            Self::Str(x) => fmt_typed_map(x, "", f),
            Self::Bytes(x) => fmt_typed_map(x, "bytes", f),
            Self::Complex(x) => fmt_typed_map(x, "complex", f),
            Self::Vec3(x) => fmt_typed_map(x, "v3", f),
            Self::Vec4(x) => fmt_typed_map(x, "v4", f),
            Self::InstantSeqEvent(x) => fmt_typed_map(x, "", f),
            Self::Volume(x) => fmt_typed_map(x, "", f),
            Self::SegmentedPhantom(x) => fmt_typed_map(x, "", f),
            Self::PhantomTissue(x) => fmt_typed_map(x, "", f),
        }
    }
}

// Helpers

struct Ellipsis(usize);

impl Debug for Ellipsis {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "... ({} more)", self.0)
    }
}

fn fmt_typed_list<T: Debug>(
    items: &[T],
    suffix: &str,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    let len = items.len();
    if len <= 10 {
        f.debug_list().entries(items).finish()?;
    } else {
        let mut list = f.debug_list();
        list.entries(&items[..8]);
        list.entry(&Ellipsis(len - 10));
        list.entries(&items[len - 2..]);
        list.finish()?;
    }
    f.write_str(suffix)
}

fn fmt_typed_map<T: Debug>(
    items: &std::collections::HashMap<String, T>,
    suffix: &str,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    let len = items.len();
    if len <= 10 {
        f.debug_map().entries(items).finish()?;
    } else {
        let mut entries = items.iter();
        let mut map = f.debug_map();
        for (k, v) in (&mut entries).take(8) {
            map.entry(k, v);
        }
        let remaining = len - 10;
        for _ in 0..remaining {
            entries.next();
        }
        map.entry(&Ellipsis(remaining), &"");
        for (k, v) in entries {
            map.entry(k, v);
        }
        map.finish()?;
    }
    f.write_str(suffix)
}
