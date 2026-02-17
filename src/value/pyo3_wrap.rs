//! `IntoPyObject` implementations for all Value types.
//!
//! Gated behind the `pyo3` feature. This allows Python bindings (e.g.
//! `toolapi-py`) to convert Rust Value types into Python objects using
//! PyO3's standard `.into_pyobject()` mechanism.

use pyo3::{
    IntoPyObjectExt,
    prelude::*,
    types::{PyDict, PyList},
};

use super::{
    Value,
    atomic::{Vec3, Vec4},
    dynamic::{Dict, List},
    structured::{InstantSeqEvent, PhantomTissue, SegmentedPhantom, Volume},
    typed::{TypedDict, TypedList},
};

// =============================================================================
// Helpers
// =============================================================================

/// Import `toolapi.value` and get a class by name.
fn value_class<'py>(py: Python<'py>, name: &str) -> PyResult<Bound<'py, PyAny>> {
    py.import("toolapi.value")?.getattr(name)
}

/// Convert a `TypedList` into a `Bound<'py, PyList>`.
fn typed_list_to_py_list<'py>(py: Python<'py>, tl: TypedList) -> PyResult<Bound<'py, PyList>> {
    match tl {
        TypedList::None(v) => {
            let l = PyList::empty(py);
            for _ in v {
                l.append(py.None())?;
            }
            Ok(l)
        }
        TypedList::Bool(v) => PyList::new(py, v),
        TypedList::Int(v) => PyList::new(py, v),
        TypedList::Float(v) => PyList::new(py, v),
        TypedList::Str(v) => PyList::new(py, v),
        TypedList::Complex(v) => PyList::new(py, v),
        TypedList::Vec3(v) => {
            let l = PyList::empty(py);
            let cls = value_class(py, "Vec3")?;
            for item in v {
                l.append(cls.call1((item.0.to_vec(),))?)?;
            }
            Ok(l)
        }
        TypedList::Vec4(v) => {
            let l = PyList::empty(py);
            let cls = value_class(py, "Vec4")?;
            for item in v {
                l.append(cls.call1((item.0.to_vec(),))?)?;
            }
            Ok(l)
        }
        TypedList::InstantSeqEvent(v) => {
            let l = PyList::empty(py);
            for item in v {
                l.append(item.into_pyobject(py)?)?;
            }
            Ok(l)
        }
        TypedList::Volume(v) => {
            let l = PyList::empty(py);
            for item in v {
                l.append(item.into_pyobject(py)?)?;
            }
            Ok(l)
        }
        TypedList::PhantomTissue(v) => {
            let l = PyList::empty(py);
            for item in v {
                l.append(item.into_pyobject(py)?)?;
            }
            Ok(l)
        }
        TypedList::SegmentedPhantom(v) => {
            let l = PyList::empty(py);
            for item in v {
                l.append(item.into_pyobject(py)?)?;
            }
            Ok(l)
        }
    }
}

// =============================================================================
// Atomic types
// =============================================================================

impl<'py> IntoPyObject<'py> for Vec3 {
    type Target = PyAny;
    type Output = Bound<'py, PyAny>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> PyResult<Self::Output> {
        let cls = value_class(py, "Vec3")?;
        cls.call1((self.0.to_vec(),))
    }
}

impl<'py> IntoPyObject<'py> for Vec4 {
    type Target = PyAny;
    type Output = Bound<'py, PyAny>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> PyResult<Self::Output> {
        let cls = value_class(py, "Vec4")?;
        cls.call1((self.0.to_vec(),))
    }
}

// =============================================================================
// Dynamic collections
// =============================================================================

impl<'py> IntoPyObject<'py> for Dict {
    type Target = PyDict;
    type Output = Bound<'py, PyDict>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> PyResult<Self::Output> {
        let dict = PyDict::new(py);
        for (key, value) in self.0 {
            let obj = value.into_pyobject(py)?;
            dict.set_item(key, obj)?;
        }
        Ok(dict)
    }
}

impl<'py> IntoPyObject<'py> for List {
    type Target = PyList;
    type Output = Bound<'py, PyList>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> PyResult<Self::Output> {
        let list = PyList::empty(py);
        for item in self.0 {
            let obj = item.into_pyobject(py)?;
            list.append(obj)?;
        }
        Ok(list)
    }
}

// =============================================================================
// Structured types
// =============================================================================

impl<'py> IntoPyObject<'py> for InstantSeqEvent {
    type Target = PyAny;
    type Output = Bound<'py, PyAny>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> PyResult<Self::Output> {
        let cls = value_class(py, "InstantSeqEvent")?;
        match self {
            InstantSeqEvent::Pulse { angle, phase } => cls.call_method1("Pulse", (angle, phase)),
            InstantSeqEvent::Fid { kt } => {
                let kt_obj = kt.into_pyobject(py)?;
                cls.call_method1("Fid", (kt_obj,))
            }
            InstantSeqEvent::Adc { phase } => cls.call_method1("Adc", (phase,)),
        }
    }
}

impl<'py> IntoPyObject<'py> for Volume {
    type Target = PyAny;
    type Output = Bound<'py, PyAny>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> PyResult<Self::Output> {
        let cls = value_class(py, "Volume")?;
        let shape = self.shape.to_vec();
        let affine: Vec<Vec<f64>> = self.affine.iter().map(|row| row.to_vec()).collect();
        let data = typed_list_to_py_list(py, self.data)?;
        cls.call1((shape, affine, data))
    }
}

impl<'py> IntoPyObject<'py> for PhantomTissue {
    type Target = PyAny;
    type Output = Bound<'py, PyAny>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> PyResult<Self::Output> {
        let cls = value_class(py, "PhantomTissue")?;
        let density = self.density.into_pyobject(py)?;
        let db0 = self.db0.into_pyobject(py)?;
        cls.call1((density, db0, self.t1, self.t2, self.t2dash, self.adc))
    }
}

impl<'py> IntoPyObject<'py> for SegmentedPhantom {
    type Target = PyAny;
    type Output = Bound<'py, PyAny>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> PyResult<Self::Output> {
        let cls = value_class(py, "SegmentedPhantom")?;
        let tissues = PyList::empty(py);
        for t in self.tissues {
            tissues.append(t.into_pyobject(py)?)?;
        }
        let b1_tx = PyList::empty(py);
        for v in self.b1_tx {
            b1_tx.append(v.into_pyobject(py)?)?;
        }
        let b1_rx = PyList::empty(py);
        for v in self.b1_rx {
            b1_rx.append(v.into_pyobject(py)?)?;
        }
        cls.call1((tissues, b1_tx, b1_rx))
    }
}

// =============================================================================
// TypedList
// =============================================================================

impl<'py> IntoPyObject<'py> for TypedList {
    type Target = PyList;
    type Output = Bound<'py, PyList>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> PyResult<Self::Output> {
        typed_list_to_py_list(py, self)
    }
}

// =============================================================================
// TypedDict
// =============================================================================

impl<'py> IntoPyObject<'py> for TypedDict {
    type Target = PyDict;
    type Output = Bound<'py, PyDict>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> PyResult<Self::Output> {
        let dict = PyDict::new(py);
        match self {
            TypedDict::None(m) => {
                for (k, _) in m {
                    dict.set_item(k, py.None())?;
                }
            }
            TypedDict::Bool(m) => {
                for (k, v) in m {
                    dict.set_item(k, v)?;
                }
            }
            TypedDict::Int(m) => {
                for (k, v) in m {
                    dict.set_item(k, v)?;
                }
            }
            TypedDict::Float(m) => {
                for (k, v) in m {
                    dict.set_item(k, v)?;
                }
            }
            TypedDict::Str(m) => {
                for (k, v) in m {
                    dict.set_item(k, v)?;
                }
            }
            TypedDict::Complex(m) => {
                for (k, v) in m {
                    dict.set_item(k, v)?;
                }
            }
            TypedDict::Vec3(m) => {
                let cls = value_class(py, "Vec3")?;
                for (k, v) in m {
                    dict.set_item(k, cls.call1((v.0.to_vec(),))?)?;
                }
            }
            TypedDict::Vec4(m) => {
                let cls = value_class(py, "Vec4")?;
                for (k, v) in m {
                    dict.set_item(k, cls.call1((v.0.to_vec(),))?)?;
                }
            }
            TypedDict::InstantSeqEvent(m) => {
                for (k, v) in m {
                    dict.set_item(k, v.into_pyobject(py)?)?;
                }
            }
            TypedDict::Volume(m) => {
                for (k, v) in m {
                    dict.set_item(k, v.into_pyobject(py)?)?;
                }
            }
            TypedDict::PhantomTissue(m) => {
                for (k, v) in m {
                    dict.set_item(k, v.into_pyobject(py)?)?;
                }
            }
            TypedDict::SegmentedPhantom(m) => {
                for (k, v) in m {
                    dict.set_item(k, v.into_pyobject(py)?)?;
                }
            }
        }
        Ok(dict)
    }
}

// =============================================================================
// Value (top-level dispatcher)
// =============================================================================

impl<'py> IntoPyObject<'py> for Value {
    type Target = PyAny;
    type Output = Bound<'py, PyAny>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> PyResult<Self::Output> {
        match self {
            Value::None(()) => Ok(py.None().into_bound(py)),
            Value::Bool(b) => b.into_bound_py_any(py),
            Value::Int(i) => i.into_bound_py_any(py),
            Value::Float(f) => f.into_bound_py_any(py),
            Value::Str(s) => s.into_bound_py_any(py),
            Value::Complex(c) => c.into_bound_py_any(py),
            Value::Vec3(v) => v.into_bound_py_any(py),
            Value::Vec4(v) => v.into_bound_py_any(py),
            Value::InstantSeqEvent(e) => e.into_bound_py_any(py),
            Value::Volume(v) => v.into_bound_py_any(py),
            Value::PhantomTissue(pt) => pt.into_bound_py_any(py),
            Value::SegmentedPhantom(sp) => sp.into_bound_py_any(py),
            Value::Dict(d) => d.into_bound_py_any(py),
            Value::List(l) => l.into_bound_py_any(py),
            Value::TypedList(tl) => tl.into_bound_py_any(py),
            Value::TypedDict(td) => td.into_bound_py_any(py),
        }
    }
}
