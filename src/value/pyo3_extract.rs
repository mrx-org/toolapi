//! `FromPyObject` implementations for all Value types.
//!
//! Gated behind the `pyo3` feature. This allows Python bindings (e.g.
//! `toolapi-py`) to convert Python objects into Rust Value types using
//! PyO3's standard `.extract()` mechanism.

use std::collections::HashMap;

use num_complex::Complex64;
use pyo3::{
    exceptions::PyTypeError,
    prelude::*,
    types::{PyDict, PyList},
};

use super::{
    atomic::{Vec3, Vec4},
    dynamic::{Dict, List},
    structured::{InstantSeqEvent, PhantomTissue, SegmentedPhantom, Volume},
    typed::TypedList,
    Value,
};

// =============================================================================
// Atomic types
// =============================================================================

impl FromPyObject<'_, '_> for Vec3 {
    type Error = PyErr;

    fn extract(obj: Borrowed<'_, '_, PyAny>) -> PyResult<Self> {
        let data: Vec<f64> = obj.getattr("data")?.extract()?;
        let arr: [f64; 3] = data
            .try_into()
            .map_err(|_| PyTypeError::new_err("Vec3.data must have 3 elements"))?;
        Ok(Vec3(arr))
    }
}

impl FromPyObject<'_, '_> for Vec4 {
    type Error = PyErr;

    fn extract(obj: Borrowed<'_, '_, PyAny>) -> PyResult<Self> {
        let data: Vec<f64> = obj.getattr("data")?.extract()?;
        let arr: [f64; 4] = data
            .try_into()
            .map_err(|_| PyTypeError::new_err("Vec4.data must have 4 elements"))?;
        Ok(Vec4(arr))
    }
}

// =============================================================================
// Dynamic collections
// =============================================================================

impl FromPyObject<'_, '_> for Dict {
    type Error = PyErr;

    fn extract(obj: Borrowed<'_, '_, PyAny>) -> PyResult<Self> {
        let dict = obj.cast::<PyDict>()?;
        let mut map = HashMap::new();
        for (key, value) in dict.iter() {
            let key: String = key.extract()?;
            let value: Value = value.extract()?;
            map.insert(key, value);
        }
        Ok(Dict(map))
    }
}

impl FromPyObject<'_, '_> for List {
    type Error = PyErr;

    fn extract(obj: Borrowed<'_, '_, PyAny>) -> PyResult<Self> {
        let list = obj.cast::<PyList>()?;
        let mut items = Vec::with_capacity(list.len());
        for item in list.iter() {
            items.push(item.extract()?);
        }
        Ok(List(items))
    }
}

// =============================================================================
// Structured types
// =============================================================================

impl FromPyObject<'_, '_> for InstantSeqEvent {
    type Error = PyErr;

    fn extract(obj: Borrowed<'_, '_, PyAny>) -> PyResult<Self> {
        let variant: String = obj.getattr("variant")?.extract()?;
        let fields = obj.getattr("fields")?;
        match variant.as_str() {
            "Pulse" => Ok(InstantSeqEvent::Pulse {
                angle: fields.get_item("angle")?.extract()?,
                phase: fields.get_item("phase")?.extract()?,
            }),
            "Fid" => {
                let kt: Vec4 = fields.get_item("kt")?.extract()?;
                Ok(InstantSeqEvent::Fid { kt })
            }
            "Adc" => Ok(InstantSeqEvent::Adc {
                phase: fields.get_item("phase")?.extract()?,
            }),
            other => Err(PyTypeError::new_err(format!(
                "unknown InstantSeqEvent variant: {other}"
            ))),
        }
    }
}

impl FromPyObject<'_, '_> for Volume {
    type Error = PyErr;

    fn extract(obj: Borrowed<'_, '_, PyAny>) -> PyResult<Self> {
        let shape_vec: Vec<u64> = obj.getattr("shape")?.extract()?;
        let shape: [u64; 3] = shape_vec
            .try_into()
            .map_err(|_| PyTypeError::new_err("Volume.shape must have 3 elements"))?;

        let affine = extract_affine(&obj.getattr("affine")?)?;
        let data: TypedList = obj.getattr("data")?.extract()?;

        Ok(Volume {
            shape,
            affine,
            data,
        })
    }
}

impl FromPyObject<'_, '_> for PhantomTissue {
    type Error = PyErr;

    fn extract(obj: Borrowed<'_, '_, PyAny>) -> PyResult<Self> {
        Ok(PhantomTissue {
            density: obj.getattr("density")?.extract()?,
            db0: obj.getattr("db0")?.extract()?,
            t1: obj.getattr("t1")?.extract()?,
            t2: obj.getattr("t2")?.extract()?,
            t2dash: obj.getattr("t2dash")?.extract()?,
            adc: obj.getattr("adc")?.extract()?,
        })
    }
}

impl FromPyObject<'_, '_> for SegmentedPhantom {
    type Error = PyErr;

    fn extract(obj: Borrowed<'_, '_, PyAny>) -> PyResult<Self> {
        Ok(SegmentedPhantom {
            tissues: obj.getattr("tissues")?.extract()?,
            b1_tx: obj.getattr("b1_tx")?.extract()?,
            b1_rx: obj.getattr("b1_rx")?.extract()?,
        })
    }
}

// =============================================================================
// TypedList (first-element heuristic)
// =============================================================================

impl FromPyObject<'_, '_> for TypedList {
    type Error = PyErr;

    fn extract(obj: Borrowed<'_, '_, PyAny>) -> PyResult<Self> {
        let list = obj.cast::<PyList>()?;
        if list.is_empty() {
            return Ok(TypedList::Float(vec![]));
        }

        let first = list.get_item(0)?;

        // Try complex before float, since complex can't extract as f64
        if first.extract::<Complex64>().is_ok() {
            let data: Vec<Complex64> = list.extract()?;
            return Ok(TypedList::Complex(data));
        }
        if first.extract::<f64>().is_ok() {
            let data: Vec<f64> = list.extract()?;
            return Ok(TypedList::Float(data));
        }
        if first.extract::<i64>().is_ok() {
            let data: Vec<i64> = list.extract()?;
            return Ok(TypedList::Int(data));
        }
        if first.extract::<bool>().is_ok() {
            let data: Vec<bool> = list.extract()?;
            return Ok(TypedList::Bool(data));
        }
        if first.extract::<String>().is_ok() {
            let data: Vec<String> = list.extract()?;
            return Ok(TypedList::Str(data));
        }

        // Structured types: check class name of first element
        if let Ok(type_name) = first.get_type().name().map(|n| n.to_string()) {
            match type_name.as_str() {
                "Vec3" => {
                    let data: Vec<Vec3> = list.extract()?;
                    return Ok(TypedList::Vec3(data));
                }
                "Vec4" => {
                    let data: Vec<Vec4> = list.extract()?;
                    return Ok(TypedList::Vec4(data));
                }
                "InstantSeqEvent" => {
                    let data: Vec<InstantSeqEvent> = list.extract()?;
                    return Ok(TypedList::InstantSeqEvent(data));
                }
                "Volume" => {
                    let data: Vec<Volume> = list.extract()?;
                    return Ok(TypedList::Volume(data));
                }
                "PhantomTissue" => {
                    let data: Vec<PhantomTissue> = list.extract()?;
                    return Ok(TypedList::PhantomTissue(data));
                }
                "SegmentedPhantom" => {
                    let data: Vec<SegmentedPhantom> = list.extract()?;
                    return Ok(TypedList::SegmentedPhantom(data));
                }
                _ => {}
            }
        }

        Err(PyTypeError::new_err(
            "cannot determine TypedList element type from list contents",
        ))
    }
}

// =============================================================================
// Value (top-level dispatcher)
// =============================================================================

impl FromPyObject<'_, '_> for Value {
    type Error = PyErr;

    fn extract(obj: Borrowed<'_, '_, PyAny>) -> PyResult<Self> {
        // None
        if obj.is_none() {
            return Ok(Value::None(()));
        }

        // Primitives: try-extract chain (order matters: bool before int)
        if let Ok(b) = obj.extract::<bool>() {
            return Ok(Value::Bool(b));
        }
        if let Ok(i) = obj.extract::<i64>() {
            return Ok(Value::Int(i));
        }
        if let Ok(f) = obj.extract::<f64>() {
            return Ok(Value::Float(f));
        }
        if let Ok(s) = obj.extract::<String>() {
            return Ok(Value::Str(s));
        }
        if let Ok(c) = obj.extract::<Complex64>() {
            return Ok(Value::Complex(c));
        }

        // Built-in collections
        if obj.is_instance_of::<PyDict>() {
            return Ok(Value::Dict(obj.extract()?));
        }
        if obj.is_instance_of::<PyList>() {
            return Ok(Value::List(obj.extract()?));
        }

        // Structured types: dispatch on class name
        let type_name = obj.get_type().name().map(|n| n.to_string()).map_err(|_| {
            PyTypeError::new_err(format!(
                "unsupported Python type for Value conversion: {}",
                obj.get_type()
            ))
        })?;

        match type_name.as_str() {
            "Vec3" => Ok(Value::Vec3(obj.extract()?)),
            "Vec4" => Ok(Value::Vec4(obj.extract()?)),
            "Volume" => Ok(Value::Volume(obj.extract()?)),
            "PhantomTissue" => Ok(Value::PhantomTissue(obj.extract()?)),
            "SegmentedPhantom" => Ok(Value::SegmentedPhantom(obj.extract()?)),
            "InstantSeqEvent" => Ok(Value::InstantSeqEvent(obj.extract()?)),
            other => Err(PyTypeError::new_err(format!(
                "unknown toolapi value type: {other}"
            ))),
        }
    }
}

// =============================================================================
// Helpers
// =============================================================================

fn extract_affine(obj: &Bound<'_, PyAny>) -> PyResult<[[f64; 4]; 3]> {
    let rows: Vec<Vec<f64>> = obj.extract()?;
    if rows.len() != 3 {
        return Err(PyTypeError::new_err("affine must have 3 rows"));
    }
    let mut affine = [[0.0f64; 4]; 3];
    for (i, row) in rows.into_iter().enumerate() {
        let arr: [f64; 4] = row
            .try_into()
            .map_err(|_| PyTypeError::new_err("each affine row must have 4 elements"))?;
        affine[i] = arr;
    }
    Ok(affine)
}
