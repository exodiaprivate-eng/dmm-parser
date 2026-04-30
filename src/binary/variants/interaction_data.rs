//! Typed structures for the InteractionInfo tail.
//!
//! Per IDA decompile of `sub_1410DFBA0` (top reader), `sub_141E2BEB0`
//! (InteractionPivot 168-byte items), `sub_141114DD0` (Optional 32-byte
//! items via sub_1410DF630), `sub_141100E90` (32-byte float items),
//! `sub_1410DF630` (32-byte InteractionInner with 2× GCO).

use crate::binary::variants::buff_data::GameConditionOptional;
use crate::binary::*;
use crate::json_traits::{ToJsonValue, WriteJsonValue, get_field as json_get_field};
use crate::py_binary_struct;
use serde_json::{Map, Value};
use std::io::{self, Write};

py_binary_struct! {
    /// One repetition of {sub_1410A9D40 CString-hash + [f32;3]} in
    /// InteractionPivot (4 of these per pivot element).
    pub struct InteractionPivotRefPoint<'a> {
        pub name: CString<'a>,
        pub position: [f32; 3],
    }
}

py_binary_struct! {
    /// sub_141E2BEB0 element (168-byte mem InteractionPivot).
    /// Wire-order reads from the && chain:
    pub struct InteractionPivot<'a> {
        pub raw_a: u32,
        pub ref_point_0: InteractionPivotRefPoint<'a>,
        pub ref_point_1: InteractionPivotRefPoint<'a>,
        pub ref_point_2: InteractionPivotRefPoint<'a>,
        pub ref_point_3: InteractionPivotRefPoint<'a>,
        pub raw_b: u32,
        pub raw_c: u32,
        pub raw_d: u32,
        pub raw_e: u32,
        pub raw_f: u32,
        pub raw_g: u32,
        pub raw_h: u32,
        pub raw_i: u32,
        pub raw_j: u32,
        pub raw_k: u32,
        pub raw_l: u32,
        pub raw_m: u32,
        pub raw_n: u32,
        pub label: CString<'a>,
        pub vec_a: [f32; 3],
        pub raw_o: u32,
        pub raw_pq: u64,
        pub raw_rs: u64,
        pub hash_113c8: u32,    // sub_141100370 u32→u16 hash
    }
}

py_binary_struct! {
    /// sub_1410DF630 element (32-byte InteractionInner). Wire:
    /// 2× GameConditionOptional + u8 + u32 hash + u32 + u8 + u8.
    pub struct InteractionInner<'a> {
        pub gc_a: GameConditionOptional<'a>,
        pub gc_b: GameConditionOptional<'a>,
        pub flag_a: u8,
        pub hash_ff050: u32,    // sub_1410FF050 u32 hash
        pub raw: u32,
        pub flag_b: u8,
        pub flag_c: u8,
    }
}

/// sub_141114DD0 element: u8 presence + (if !=0) InteractionInner.
#[derive(Debug)]
pub struct OptionalInteractionInner<'a> {
    pub presence: u8,
    pub inner: Option<InteractionInner<'a>>,
}

impl<'a> BinaryRead<'a> for OptionalInteractionInner<'a> {
    fn read_from(data: &'a [u8], offset: &mut usize) -> io::Result<Self> {
        let presence = u8::read_from(data, offset)?;
        let inner = if presence != 0 {
            Some(InteractionInner::read_from(data, offset)?)
        } else { None };
        Ok(Self { presence, inner })
    }
}

impl<'a> BinaryWrite for OptionalInteractionInner<'a> {
    fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        self.presence.write_to(w)?;
        if let Some(i) = &self.inner { i.write_to(w)?; }
        Ok(())
    }
}

impl<'a> BinaryReadTracked<'a> for OptionalInteractionInner<'a> {
    fn read_tracked(data: &'a [u8], offset: &mut usize, _p: &mut String, _r: &mut Vec<FieldRange>)
        -> io::Result<Self>
    { <Self as BinaryRead>::read_from(data, offset) }
}

impl<'a> ToJsonValue for OptionalInteractionInner<'a> {
    fn to_json_value(&self) -> Value {
        let mut m = Map::new();
        m.insert("presence".into(), self.presence.to_json_value());
        m.insert("inner".into(), match &self.inner {
            Some(i) => i.to_json_value(), None => Value::Null,
        });
        Value::Object(m)
    }
}

impl<'a> WriteJsonValue for OptionalInteractionInner<'a> {
    fn write_from_json(w: &mut Vec<u8>, v: &Value) -> io::Result<()> {
        let obj = v.as_object().ok_or_else(|| io::Error::new(
            io::ErrorKind::InvalidData, "OptionalInteractionInner: expected object"))?;
        let presence = json_get_field(obj, "presence")?.as_u64()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData,
                "OptionalInteractionInner.presence: expected u8"))? as u8;
        w.push(presence);
        if presence != 0 {
            <InteractionInner as WriteJsonValue>::write_from_json(
                w, json_get_field(obj, "inner")?)?;
        }
        Ok(())
    }
}

impl<'a> crate::python_traits::ToPyValue for OptionalInteractionInner<'a> {
    fn to_py_value(&self, _py: pyo3::Python<'_>) -> pyo3::PyResult<pyo3::Py<pyo3::PyAny>> {
        Err(pyo3::exceptions::PyNotImplementedError::new_err("use JSON"))
    }
}

impl<'a> crate::python_traits::WritePyValue for OptionalInteractionInner<'a> {
    fn write_from_py(_w: &mut Vec<u8>, _o: &pyo3::Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<()> {
        Err(pyo3::exceptions::PyNotImplementedError::new_err("use JSON"))
    }
}

py_binary_struct! {
    /// sub_141100E90 element (32-byte mem). Wire: f32 + 6×4 = 28 bytes.
    /// Likely a 7-component float vector.
    pub struct InteractionFloat32 {
        pub a: u32,
        pub b: u32,
        pub c: u32,
        pub d: u32,
        pub e: u32,
        pub f: u32,
        pub g: u32,
    }
}
