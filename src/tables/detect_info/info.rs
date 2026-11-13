//! Hand-corrected: IDA-derived parser for `DetectInfo.pabgb`.
//!
//! Per IDA sub_1410D9E36 (parser) + sub_1410D9B70 (DetectSenseData reader).
//! DetectSenseData is a RECURSIVE tree: it contains a list of children
//! that are themselves DetectSenseData. Use Box<CArray<...>> indirection
//! since CArray<T> already heap-allocates the element backing.


// ─────────────────────────────────────────────────────────────────────────
// CANONICAL FIELD CATALOG — pa::DetectInfo
// ─────────────────────────────────────────────────────────────────────────
//
// Schema source: NattKh/CrimsonDesertModdingTools `pabgb_complete_schema.json`
// (canonical PA names extracted from Korean error strings in CrimsonDesert.exe).
//
// Total canonical fields:  7
// Decoded by dmm-parser:   7
// Missing in this struct:  0
//
// ✅ = present in this struct (round-trips via shape='v3.1')
// ⏳ = in canonical schema but not yet decoded by dmm-parser
//
// ✅ _isDetectableAttachedDocking (direct_u8, stream=1)
// ✅ _decreaseValuePerSec (direct_u32, stream=4)
// ✅ _detectSenseData (array_or_complex, stream=4)
// ✅ _targetLostDistance (direct_u32, stream=4)
// ✅ _key (array_or_complex, stream=4)
// ✅ _isBlocked (direct_u8, stream=1)
// ✅ _stringKey

use crate::binary::*;
use crate::py_binary_struct;

py_binary_struct! {
    pub struct DetectSenseData {
        pub field_a: u32,
        pub field_b: u32,
        pub field_c: u32,
        pub field_d: u32,
        pub field_e: u32,
        pub field_f: u32,
        pub field_g: u32,
        pub field_h: u32,
        // 2.04.00: `_globalDetectTargetDistance` moved here from DetectInfo (Mac reader
        // sub_102072234, the ninth float before the two flags).
        pub global_detect_target_distance: u32,
        pub flag_a: u8,
        pub flag_b: u8,
        pub children: CArray<DetectSenseData>,
    }
}

py_binary_struct! {
    // 2.04.00 order (Mac reader sub_1020724E8). New: `_underWaterDetectionType`; the 2.03
    // `_globalDetectTargetDistance` moved into DetectSenseData.
    pub struct DetectInfo<'a> {
        pub string_key: CString<'a>,
        pub key: u16,
        pub is_blocked: u8,
        pub under_water_detection_type: u8,
        pub is_detectable_attached_docking: u8,
        pub decrease_value_per_sec: u32,
        pub target_lost_distance: u32,
        pub detect_sense_data: DetectSenseData,
        // 2.05.00: `_detectDetailData` (field 8). The DetectDetailInfo table is gone; its 64-entry
        // list now sits in each DetectInfo record behind a presence byte (25 records 0, 19 records
        // 1 + exactly the old table's 2688 / 2982 bytes).
        pub detect_detail_data: COptional<DetectDetailList>,
    }
}

/// The 64 DetectSenseData entries of 2.05's `_detectDetailData` (`_detectDetailDataListNew`): a
/// FIXED count with no prefix, as the old DetectDetailInfo table stored them.
#[derive(Debug)]
pub struct DetectDetailList(pub Vec<DetectSenseData>);

pub const DETECT_DETAIL_LEN: usize = 0x40;

impl<'a> BinaryRead<'a> for DetectDetailList {
    fn read_from(data: &'a [u8], offset: &mut usize) -> std::io::Result<Self> {
        let mut v = Vec::with_capacity(DETECT_DETAIL_LEN);
        for _ in 0..DETECT_DETAIL_LEN {
            v.push(DetectSenseData::read_from(data, offset)?);
        }
        Ok(Self(v))
    }
}

impl<'a> BinaryReadTracked<'a> for DetectDetailList {
    fn read_tracked(
        data: &'a [u8],
        offset: &mut usize,
        path: &mut String,
        ranges: &mut Vec<FieldRange>,
    ) -> std::io::Result<Self> {
        let mut v = Vec::with_capacity(DETECT_DETAIL_LEN);
        for i in 0..DETECT_DETAIL_LEN {
            let saved = push_path(path, &i.to_string());
            v.push(DetectSenseData::read_tracked(data, offset, path, ranges)?);
            pop_path(path, saved);
        }
        Ok(Self(v))
    }
}

impl BinaryWrite for DetectDetailList {
    fn write_to(&self, w: &mut dyn std::io::Write) -> std::io::Result<()> {
        if self.0.len() != DETECT_DETAIL_LEN {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("detect_detail_data must have exactly {} entries (got {})", DETECT_DETAIL_LEN, self.0.len()),
            ));
        }
        for e in &self.0 {
            e.write_to(w)?;
        }
        Ok(())
    }
}

impl crate::json_traits::ToJsonValue for DetectDetailList {
    fn to_json_value(&self) -> serde_json::Value {
        serde_json::Value::Array(self.0.iter().map(|e| e.to_json_value()).collect())
    }
}

impl crate::json_traits::WriteJsonValue for DetectDetailList {
    fn write_from_json(w: &mut Vec<u8>, v: &serde_json::Value) -> std::io::Result<()> {
        let arr = v.as_array().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, "detect_detail_data: expected an array")
        })?;
        if arr.len() != DETECT_DETAIL_LEN {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("detect_detail_data must have exactly {} entries (got {})", DETECT_DETAIL_LEN, arr.len()),
            ));
        }
        for e in arr {
            <DetectSenseData as crate::json_traits::WriteJsonValue>::write_from_json(w, e)?;
        }
        Ok(())
    }
}

impl crate::python_traits::ToPyValue for DetectDetailList {
    fn to_py_value(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<pyo3::Py<pyo3::PyAny>> {
        use pyo3::types::PyListMethods;
        let list = pyo3::types::PyList::empty(py);
        for e in &self.0 {
            list.append(e.to_py_value(py)?)?;
        }
        Ok(list.into_any().unbind())
    }
}

impl crate::python_traits::WritePyValue for DetectDetailList {
    fn write_from_py(w: &mut Vec<u8>, obj: &pyo3::Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<()> {
        use pyo3::types::PyListMethods;
        let list = obj.cast::<pyo3::types::PyList>()?;
        if list.len() != DETECT_DETAIL_LEN {
            return Err(pyo3::exceptions::PyValueError::new_err(format!(
                "detect_detail_data must have exactly {} entries (got {})",
                DETECT_DETAIL_LEN,
                list.len()
            )));
        }
        for item in list.iter() {
            <DetectSenseData as crate::python_traits::WritePyValue>::write_from_py(w, &item)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pabgb_path() -> std::path::PathBuf { crate::testenv::resolve("detectinfo.pabgb") }
#[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(pabgb_path()) else {
            eprintln!("SKIP: fixture not found");
            return;
        };
        let mut offset = 0;
        let mut items = Vec::new();
        while offset < data.len() {
            items.push(DetectInfo::read_from(&data, &mut offset).unwrap());
        }
        assert_eq!(offset, data.len(), "did not consume all bytes");
        let mut out = Vec::with_capacity(data.len());
        for item in &items {
            item.write_to(&mut out).unwrap();
        }
        assert_eq!(out, data, "detectinfo roundtrip bytes mismatch");
    }

    #[test]
    fn json_roundtrip() {
        let Ok(data) = std::fs::read(pabgb_path()) else {
            eprintln!("SKIP: fixture not found");
            return;
        };
        let mut offset = 0;
        let mut items = Vec::new();
        while offset < data.len() {
            items.push(DetectInfo::read_from(&data, &mut offset).unwrap());
        }
        assert_eq!(offset, data.len(), "did not consume all bytes");

        for (i, item) in items.iter().enumerate() {
            let _ = &item;
            let dict = item.to_json_dict();
            let mut from_typed = Vec::new();
            item.write_to(&mut from_typed).unwrap();
            let mut from_json = Vec::new();
            DetectInfo::write_from_json_dict(&mut from_json, &dict)
                .unwrap_or_else(|e| panic!("entry {} key=0x{:x}: write_from_json_dict: {}", i, item.key, e));
            assert_eq!(
                from_json, from_typed,
                "entry {} key=0x{:x}: JSON round-trip diverges from typed write",
                i, item.key
            );
        }
    }
}
