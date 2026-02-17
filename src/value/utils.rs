use crate::value::typed::TypedList;

impl TypedList {
    pub fn is_empty(&self) -> bool {
        match self {
            TypedList::None(items) => items.is_empty(),
            TypedList::Bool(items) => items.is_empty(),
            TypedList::Int(items) => items.is_empty(),
            TypedList::Float(items) => items.is_empty(),
            TypedList::Complex(items) => items.is_empty(),
            TypedList::Vec3(items) => items.is_empty(),
            TypedList::Vec4(items) => items.is_empty(),
            TypedList::Str(items) => items.is_empty(),
            TypedList::InstantSeqEvent(items) => items.is_empty(),
            TypedList::Volume(items) => items.is_empty(),
            TypedList::SegmentedPhantom(items) => items.is_empty(),
            TypedList::PhantomTissue(items) => items.is_empty(),
        }
    }
}
