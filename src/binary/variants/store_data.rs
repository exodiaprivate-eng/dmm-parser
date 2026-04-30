//! Typed structures for the StoreInfo tail (sub_1410FC8F0 stockData
//! element + trailers).
//!
//! Per IDA decompile this iteration. sub_141D03AA0 reuses the existing
//! `OptionalDropTarget` from `random_box_item.rs`.

use crate::binary::variants::drop_target::OptionalDropTarget;
use crate::binary::*;
use crate::json_traits::{ToJsonValue, WriteJsonValue, get_field as json_get_field};
use crate::py_binary_struct;
use serde_json::{Map, Value};
use std::io::{self, Write};

py_binary_struct! {
    /// sub_1410D9E90 inner: u32 hash (sub_1410FF430) + u8 + u32 hash
    /// (sub_1410FF050) + u32 hash (sub_1410FF050). Wire: 13 bytes.
    pub struct StoreStockOptionalInner {
        pub hash_a: u32,
        pub flag: u8,
        pub hash_b: u32,
        pub hash_c: u32,
    }
}

/// u8 presence + (if !=0: StoreStockOptionalInner).
#[derive(Debug)]
pub struct StoreStockOptional {
    pub presence: u8,
    pub inner: Option<StoreStockOptionalInner>,
}

impl<'a> BinaryRead<'a> for StoreStockOptional {
    fn read_from(data: &'a [u8], offset: &mut usize) -> io::Result<Self> {
        let presence = u8::read_from(data, offset)?;
        let inner = if presence != 0 {
            Some(StoreStockOptionalInner::read_from(data, offset)?)
        } else { None };
        Ok(Self { presence, inner })
    }
}

impl BinaryWrite for StoreStockOptional {
    fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        self.presence.write_to(w)?;
        if let Some(i) = &self.inner { i.write_to(w)?; }
        Ok(())
    }
}

impl<'a> BinaryReadTracked<'a> for StoreStockOptional {
    fn read_tracked(data: &'a [u8], offset: &mut usize, _p: &mut String, _r: &mut Vec<FieldRange>)
        -> io::Result<Self>
    { <Self as BinaryRead>::read_from(data, offset) }
}

impl ToJsonValue for StoreStockOptional {
    fn to_json_value(&self) -> Value {
        let mut m = Map::new();
        m.insert("presence".into(), self.presence.to_json_value());
        m.insert("inner".into(), match &self.inner {
            Some(i) => i.to_json_value(), None => Value::Null,
        });
        Value::Object(m)
    }
}

impl WriteJsonValue for StoreStockOptional {
    fn write_from_json(w: &mut Vec<u8>, v: &Value) -> io::Result<()> {
        let obj = v.as_object().ok_or_else(|| io::Error::new(
            io::ErrorKind::InvalidData, "StoreStockOptional: expected object"))?;
        let presence = json_get_field(obj, "presence")?.as_u64()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData,
                "StoreStockOptional.presence: expected u8"))? as u8;
        w.push(presence);
        if presence != 0 {
            <StoreStockOptionalInner as WriteJsonValue>::write_from_json(
                w, json_get_field(obj, "inner")?)?;
        }
        Ok(())
    }
}

impl crate::python_traits::ToPyValue for StoreStockOptional {
    fn to_py_value(&self, _py: pyo3::Python<'_>) -> pyo3::PyResult<pyo3::Py<pyo3::PyAny>> {
        Err(pyo3::exceptions::PyNotImplementedError::new_err("use JSON"))
    }
}

impl crate::python_traits::WritePyValue for StoreStockOptional {
    fn write_from_py(_w: &mut Vec<u8>, _o: &pyo3::Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<()> {
        Err(pyo3::exceptions::PyNotImplementedError::new_err("use JSON"))
    }
}

py_binary_struct! {
    /// CArray-of-pairs at trailing of stock data. Wire: u32 hash + u64.
    pub struct StoreHashU64 {
        pub hash: u32,
        pub raw: u64,
    }
}

py_binary_struct! {
    /// sub_1410FC8F0 stockData element. Wire layout:
    ///   - sub_141103610 u16 hash (2 wire bytes) at +8
    ///   - u64 raw (8) at +16
    ///   - u64 raw (8) at +24
    ///   - u32 (4) at +32
    ///   - u32 (4) at +36
    ///   - u32 (4) at +40
    ///   - u8 + u8 + u8 at +44/+45/+46
    ///   - OptionalDropTarget (sub_141D03AA0) at +48
    ///   - u32 hash (sub_1410FF430) at +56 (4 wire bytes)
    ///   - StoreStockOptional (presence + inner) at +60
    ///   - CArray<StoreHashU64> at +72
    pub struct StoreStockData {
        pub hash_115e8: u16,         // sub_141103610 (2 wire bytes)
        pub raw_u64_a: u64,
        pub raw_u64_b: u64,
        pub raw_a: u32,
        pub raw_b: u32,
        pub raw_c: u32,
        pub flag_a: u8,
        pub flag_b: u8,
        pub flag_c: u8,
        pub random_box: OptionalDropTarget,
        pub hash_e9c0: u32,           // sub_1410FF430 (4 wire bytes)
        pub optional: StoreStockOptional,
        pub hash_u64_list: CArray<StoreHashU64>,
    }
}
