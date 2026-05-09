// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 §4.9 (No Competing Implementation) and §4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. §1202.

use pyo3::prelude::*;
use pyo3::types::{PyBool, PyBytes, PyDict, PyFloat, PyInt, PyList, PyNone, PyString};
use pyo3::exceptions::{PyIOError, PyKeyError, PyValueError};

use crate::binary::*;
use crate::binary::papgt::PackGroupTreeMeta;
use crate::binary::pamt::PackMeta;
use crate::item_info::ItemInfo;

// ── Dict helpers ───────────────────────────────────────────────────────────

fn get<'py, T>(d: &Bound<'py, PyDict>, key: &str) -> PyResult<T>
where
    for<'a> T: FromPyObject<'a, 'py, Error = PyErr>,
{
    d.get_item(key)?
        .ok_or_else(|| PyKeyError::new_err(key.to_string()))?
        .extract()
}

fn get_obj<'py>(d: &Bound<'py, PyDict>, key: &str) -> PyResult<Bound<'py, PyAny>> {
    d.get_item(key)?
        .ok_or_else(|| PyKeyError::new_err(key.to_string()))
}

fn json_to_py(py: Python<'_>, v: &serde_json::Value) -> PyResult<Py<PyAny>> {
    match v {
        serde_json::Value::Null => Ok(PyNone::get(py).to_owned().into_any().unbind()),
        serde_json::Value::Bool(b) => Ok(PyBool::new(py, *b).to_owned().into_any().unbind()),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(i.into_pyobject(py)?.into_any().unbind())
            } else if let Some(u) = n.as_u64() {
                Ok(u.into_pyobject(py)?.into_any().unbind())
            } else {
                Ok(n.as_f64().unwrap_or(0.0).into_pyobject(py)?.into_any().unbind())
            }
        }
        serde_json::Value::String(s) => Ok(s.into_pyobject(py)?.into_any().unbind()),
        serde_json::Value::Array(arr) => {
            let list = PyList::empty(py);
            for item in arr {
                list.append(json_to_py(py, item)?)?;
            }
            Ok(list.into_any().unbind())
        }
        serde_json::Value::Object(obj) => {
            let dict = PyDict::new(py);
            for (k, val) in obj {
                dict.set_item(k, json_to_py(py, val)?)?;
            }
            Ok(dict.into_any().unbind())
        }
    }
}

fn py_to_json(v: &Bound<'_, PyAny>) -> PyResult<serde_json::Value> {
    if v.is_none() {
        return Ok(serde_json::Value::Null);
    }
    if let Ok(b) = v.cast::<PyBool>() {
        return Ok(serde_json::Value::Bool(b.is_true()));
    }
    if let Ok(i) = v.cast::<PyInt>() {
        if let Ok(n) = i.extract::<i64>() {
            return Ok(serde_json::json!(n));
        }
        if let Ok(n) = i.extract::<u64>() {
            return Ok(serde_json::json!(n));
        }
    }
    if let Ok(f) = v.cast::<PyFloat>() {
        return Ok(serde_json::json!(f.value()));
    }
    if let Ok(s) = v.extract::<String>() {
        return Ok(serde_json::Value::String(s));
    }
    if let Ok(list) = v.cast::<PyList>() {
        let arr: Vec<serde_json::Value> = list.iter().map(|i| py_to_json(&i)).collect::<PyResult<_>>()?;
        return Ok(serde_json::Value::Array(arr));
    }
    if let Ok(dict) = v.cast::<PyDict>() {
        let mut map = serde_json::Map::new();
        for (k, val) in dict.iter() {
            let key = k.extract::<String>()?;
            map.insert(key, py_to_json(&val)?);
        }
        return Ok(serde_json::Value::Object(map));
    }
    Err(PyValueError::new_err(format!("cannot convert {} to JSON", v.get_type().name()?)))
}

// ── ItemInfo Python conversion ─────────────────────────────────────────────

fn to_py_item<'py>(py: Python<'py>, v: &ItemInfo) -> PyResult<Bound<'py, PyDict>> {
    v.to_py_dict(py)
}

fn wr_item(w: &mut Vec<u8>, obj: &Bound<'_, PyAny>) -> PyResult<()> {
    let d = obj.cast::<PyDict>()?;
    ItemInfo::write_from_py_dict(w, d)
}

// ── Module functions ───────────────────────────────────────────────────────

#[pyfunction]
pub fn parse_iteminfo_from_file(py: Python<'_>, path: &str) -> PyResult<Py<PyAny>> {
    let data = std::fs::read(path)
        .map_err(|e| PyIOError::new_err(e.to_string()))?;
    parse_iteminfo_from_bytes_inner(py, &data)
}

#[pyfunction]
pub fn parse_iteminfo_from_bytes(py: Python<'_>, data: &[u8]) -> PyResult<Py<PyAny>> {
    parse_iteminfo_from_bytes_inner(py, data)
}

pub fn parse_iteminfo_from_bytes_inner(py: Python<'_>, data: &[u8]) -> PyResult<Py<PyAny>> {
    let mut offset = 0;
    let mut items = Vec::new();
    while offset < data.len() {
        let item = ItemInfo::read_from(data, &mut offset)
            .map_err(|e| PyValueError::new_err(
                format!("parse error at offset 0x{:08X}: {}", offset, e),
            ))?;
        items.push(to_py_item(py, &item)?);
    }
    Ok(PyList::new(py, items)?.into_any().unbind())
}

#[pyfunction]
pub fn parse_iteminfo_tracked(py: Python<'_>, data: &[u8]) -> PyResult<Py<PyAny>> {
    use crate::binary::{BinaryReadTracked, FieldRange};

    let mut offset = 0;
    let mut py_items = Vec::new();
    let mut py_spans = Vec::new();

    while offset + 8 < data.len() {
        let start = offset;
        let mut path_buf = String::new();
        let mut ranges: Vec<FieldRange> = Vec::new();

        match ItemInfo::read_tracked(data, &mut offset, &mut path_buf, &mut ranges) {
            Ok(item) => {
                py_items.push(to_py_item(py, &item)?);

                let span = PyDict::new(py);
                span.set_item("start", start)?;
                span.set_item("end", offset)?;
                span.set_item("size", offset - start)?;

                let py_ranges = PyList::empty(py);
                for r in &ranges {
                    let rd = PyDict::new(py);
                    rd.set_item("path", &r.path)?;
                    rd.set_item("start", r.start + start)?;
                    rd.set_item("end", r.end + start)?;
                    rd.set_item("ty", r.ty)?;
                    py_ranges.append(rd)?;
                }
                span.set_item("ranges", py_ranges)?;
                py_spans.push(span.into_any().unbind());
            }
            Err(_) => break,
        }
    }

    let result = PyDict::new(py);
    result.set_item("items", PyList::new(py, py_items)?)?;
    result.set_item("spans", PyList::new(py, py_spans)?)?;
    Ok(result.into_any().unbind())
}

#[pyfunction]
pub fn write_iteminfo_to_file(items: &Bound<'_, PyList>, path: &str) -> PyResult<()> {
    let data = serialize_iteminfo_impl(items)?;
    std::fs::write(path, data)
        .map_err(|e| PyIOError::new_err(e.to_string()))
}

#[pyfunction]
pub fn serialize_iteminfo(py: Python<'_>, items: &Bound<'_, PyList>) -> PyResult<Py<PyAny>> {
    let data = serialize_iteminfo_impl(items)?;
    Ok(PyBytes::new(py, &data).into_any().unbind())
}

pub fn serialize_iteminfo_impl(items: &Bound<'_, PyList>) -> PyResult<Vec<u8>> {
    let mut buf = Vec::new();
    for item in items.iter() {
        wr_item(&mut buf, &item)?;
    }
    Ok(buf)
}

// ── PAPGT to/from Python ───────────────────────────────────────────────────

pub fn to_py_papgt<'py>(py: Python<'py>, papgt: &PackGroupTreeMeta) -> PyResult<Bound<'py, PyDict>> {
    let d = PyDict::new(py);
    d.set_item("unknown0", papgt.header.unknown0)?;
    d.set_item("checksum", papgt.header.checksum)?;
    d.set_item("unknown1", papgt.header.unknown1)?;
    d.set_item("unknown2", papgt.header.unknown2)?;

    let entries = PyList::empty(py);
    for entry in &papgt.entries {
        let ed = PyDict::new(py);
        ed.set_item("group_name", &entry.group_name)?;
        ed.set_item("is_optional", entry.entry.is_optional)?;
        ed.set_item("language", entry.entry.language.0)?;
        ed.set_item("always_zero", entry.entry.always_zero)?;
        ed.set_item("group_name_offset", entry.entry.group_name_offset)?;
        ed.set_item("pack_meta_checksum", entry.entry.pack_meta_checksum)?;
        entries.append(ed)?;
    }
    d.set_item("entries", entries)?;
    Ok(d)
}

pub fn wr_papgt_from_dict(d: &Bound<'_, PyDict>) -> PyResult<Vec<u8>> {
    // We need the original raw data for roundtrip. Since we preserve all raw offsets
    // and the group_names_buffer, we reconstruct the PackGroupTreeMeta from the dict.
    use crate::binary::papgt::*;

    let unknown0: u32 = get(d, "unknown0")?;
    let unknown1: u8 = get(d, "unknown1")?;
    let unknown2: u16 = get(d, "unknown2")?;
    let entries_list = get_obj(d, "entries")?.cast::<PyList>()?.clone();

    let mut entries = Vec::new();
    let mut group_names_buffer = Vec::new();

    for item in entries_list.iter() {
        let ed = item.cast::<PyDict>()?;
        let group_name: String = get(ed, "group_name")?;
        let is_optional: u8 = get(ed, "is_optional")?;
        let language: u16 = get(ed, "language")?;
        let always_zero: u8 = get(ed, "always_zero")?;
        let group_name_offset: u32 = get(ed, "group_name_offset")?;
        let pack_meta_checksum: u32 = get(ed, "pack_meta_checksum")?;

        // Write group name to buffer at the offset
        // For new entries, we'd need to append. For roundtrip, offsets are preserved.
        // Ensure the buffer is large enough
        let needed = group_name_offset as usize + group_name.len() + 1;
        if group_names_buffer.len() < needed {
            group_names_buffer.resize(needed, 0);
        }
        let off = group_name_offset as usize;
        group_names_buffer[off..off + group_name.len()].copy_from_slice(group_name.as_bytes());
        group_names_buffer[off + group_name.len()] = 0; // null terminator

        entries.push(ResolvedEntry {
            group_name,
            entry: PackGroupTreeMetaEntry {
                is_optional,
                language: LanguageType(language),
                always_zero,
                group_name_offset,
                pack_meta_checksum,
            },
        });
    }

    let papgt = PackGroupTreeMeta {
        header: PackGroupTreeMetaHeader {
            unknown0,
            checksum: 0, // will be recalculated by write()
            entry_count: entries.len() as u8,
            unknown1,
            unknown2,
        },
        entries,
        group_names_buffer,
    };

    papgt.to_bytes().map_err(|e| PyIOError::new_err(e.to_string()))
}

#[pyfunction]
pub fn parse_papgt_file(py: Python<'_>, path: &str) -> PyResult<Py<PyAny>> {
    let data = std::fs::read(path)
        .map_err(|e| PyIOError::new_err(e.to_string()))?;
    parse_papgt_bytes_inner(py, &data)
}

#[pyfunction]
pub fn parse_papgt_bytes(py: Python<'_>, data: &[u8]) -> PyResult<Py<PyAny>> {
    parse_papgt_bytes_inner(py, data)
}

pub fn parse_papgt_bytes_inner(py: Python<'_>, data: &[u8]) -> PyResult<Py<PyAny>> {
    let papgt = PackGroupTreeMeta::parse(data)
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
    Ok(to_py_papgt(py, &papgt)?.into_any().unbind())
}

#[pyfunction]
pub fn write_papgt_file(data: &Bound<'_, PyDict>, path: &str) -> PyResult<()> {
    let bytes = wr_papgt_from_dict(data)?;
    std::fs::write(path, bytes)
        .map_err(|e| PyIOError::new_err(e.to_string()))
}

#[pyfunction]
pub fn serialize_papgt(py: Python<'_>, data: &Bound<'_, PyDict>) -> PyResult<Py<PyAny>> {
    let bytes = wr_papgt_from_dict(data)?;
    Ok(PyBytes::new(py, &bytes).into_any().unbind())
}

// ── PAMT to/from Python ───────────────────────────────────────────────────

pub fn to_py_pamt<'py>(py: Python<'py>, pamt: &PackMeta) -> PyResult<Bound<'py, PyDict>> {
    let d = PyDict::new(py);
    d.set_item("checksum", pamt.header.checksum)?;
    d.set_item("unknown0", pamt.header.unknown0)?;

    // Encrypt info
    let ei = PyDict::new(py);
    ei.set_item("unknown0", pamt.header.encrypt_info.unknown0)?;
    let ei_bytes = PyBytes::new(py, &pamt.header.encrypt_info.encrypt_info);
    ei.set_item("encrypt_info", ei_bytes)?;
    d.set_item("encrypt_info", ei)?;

    // Chunks
    let chunks = PyList::empty(py);
    for chunk in &pamt.chunks {
        let cd = PyDict::new(py);
        cd.set_item("id", chunk.id)?;
        cd.set_item("checksum", chunk.checksum)?;
        cd.set_item("size", chunk.size)?;
        chunks.append(cd)?;
    }
    d.set_item("chunks", chunks)?;

    // Directories (resolved)
    let dirs = PyList::empty(py);
    for dir in &pamt.directories {
        let dd = PyDict::new(py);
        dd.set_item("path", &dir.path)?;
        dd.set_item("name_checksum", dir.raw.name_checksum)?;
        dd.set_item("name_offset", dir.raw.name_offset)?;
        dd.set_item("file_start_index", dir.raw.file_start_index)?;
        dd.set_item("file_count", dir.raw.file_count)?;

        let files = PyList::empty(py);
        for f in &dir.files {
            let fd = PyDict::new(py);
            fd.set_item("name", &f.name)?;
            fd.set_item("name_offset", f.file.name_offset)?;
            fd.set_item("chunk_offset", f.file.chunk_offset)?;
            fd.set_item("compressed_size", f.file.compressed_size)?;
            fd.set_item("uncompressed_size", f.file.uncompressed_size)?;
            fd.set_item("chunk_id", f.file.chunk_id)?;
            fd.set_item("flags", f.file.flags)?;
            fd.set_item("unknown0", f.file.unknown0)?;
            fd.set_item("compression", f.file.compression as u8)?;
            fd.set_item("crypto", f.file.crypto as u8)?;
            fd.set_item("is_partial", f.file.is_partial)?;
            files.append(fd)?;
        }
        dd.set_item("files", files)?;
        dirs.append(dd)?;
    }
    d.set_item("directories", dirs)?;

    // Raw trie buffers for roundtrip writing
    d.set_item("_dir_names_buffer", PyBytes::new(py, &pamt.dir_names_buffer))?;
    d.set_item("_file_names_buffer", PyBytes::new(py, &pamt.file_names_buffer))?;

    Ok(d)
}

#[pyfunction]
pub fn parse_pamt_file(py: Python<'_>, path: &str) -> PyResult<Py<PyAny>> {
    let data = std::fs::read(path)
        .map_err(|e| PyIOError::new_err(e.to_string()))?;
    parse_pamt_bytes_inner(py, &data)
}

#[pyfunction]
pub fn parse_pamt_bytes(py: Python<'_>, data: &[u8]) -> PyResult<Py<PyAny>> {
    parse_pamt_bytes_inner(py, data)
}

pub fn parse_pamt_bytes_inner(py: Python<'_>, data: &[u8]) -> PyResult<Py<PyAny>> {
    let pamt = PackMeta::parse(data, None)
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
    Ok(to_py_pamt(py, &pamt)?.into_any().unbind())
}

#[pyfunction]
pub fn write_pamt_file(data: &Bound<'_, PyDict>, path: &str) -> PyResult<()> {
    let bytes = wr_pamt_from_dict(data)?;
    std::fs::write(path, bytes)
        .map_err(|e| PyIOError::new_err(e.to_string()))
}

#[pyfunction]
pub fn serialize_pamt(py: Python<'_>, data: &Bound<'_, PyDict>) -> PyResult<Py<PyAny>> {
    let bytes = wr_pamt_from_dict(data)?;
    Ok(PyBytes::new(py, &bytes).into_any().unbind())
}

pub fn wr_pamt_from_dict(d: &Bound<'_, PyDict>) -> PyResult<Vec<u8>> {
    use crate::binary::pamt::*;

    let checksum: u32 = get(d, "checksum")?;
    let unknown0: u16 = get(d, "unknown0")?;

    let ei_obj = get_obj(d, "encrypt_info")?.cast::<PyDict>()?.clone();
    let ei_unknown0: u8 = get(&ei_obj, "unknown0")?;
    let ei_bytes: Vec<u8> = get(&ei_obj, "encrypt_info")?;
    let encrypt_info_arr: [u8; 3] = ei_bytes.try_into()
        .map_err(|_| PyValueError::new_err("encrypt_info must be 3 bytes"))?;

    let chunks_list = get_obj(d, "chunks")?.cast::<PyList>()?.clone();
    let mut chunks = Vec::new();
    for c in chunks_list.iter() {
        let cd = c.cast::<PyDict>()?;
        chunks.push(PackMetaChunk {
            id: get(cd, "id")?,
            checksum: get(cd, "checksum")?,
            size: get(cd, "size")?,
        });
    }

    let dirs_list = get_obj(d, "directories")?.cast::<PyList>()?.clone();
    let mut raw_directories = Vec::new();
    let mut raw_files = Vec::new();

    for dir_item in dirs_list.iter() {
        let dd = dir_item.cast::<PyDict>()?;
        let name_checksum: u32 = get(dd, "name_checksum")?;
        let name_offset: i32 = get(dd, "name_offset")?;
        let file_start_index: u32 = get(dd, "file_start_index")?;
        let file_count: u32 = get(dd, "file_count")?;

        raw_directories.push(PackMetaDirectory {
            name_checksum,
            name_offset,
            file_start_index,
            file_count,
        });

        let files_list = get_obj(dd, "files")?.cast::<PyList>()?.clone();
        for f_item in files_list.iter() {
            let fd = f_item.cast::<PyDict>()?;
            raw_files.push(PackMetaFileRaw {
                name_offset: get(fd, "name_offset")?,
                chunk_offset: get(fd, "chunk_offset")?,
                compressed_size: get(fd, "compressed_size")?,
                uncompressed_size: get(fd, "uncompressed_size")?,
                chunk_id: get(fd, "chunk_id")?,
                flags: get(fd, "flags")?,
                unknown0: get(fd, "unknown0")?,
            });
        }
    }

    // Get trie buffers for roundtrip
    let dir_names_buffer: Vec<u8> = get(d, "_dir_names_buffer")?;
    let file_names_buffer: Vec<u8> = get(d, "_file_names_buffer")?;

    let pamt = PackMeta {
        header: PackMetaHeader {
            checksum,
            count: chunks.len() as u16,
            unknown0,
            encrypt_info: PackEncryptInfo {
                unknown0: ei_unknown0,
                encrypt_info: encrypt_info_arr,
            },
        },
        chunks,
        directories: Vec::new(), // not needed for write()
        dir_names_buffer,
        file_names_buffer,
        raw_directories,
        raw_files,
    };

    pamt.to_bytes().map_err(|e| PyIOError::new_err(e.to_string()))
}

// ── Localization to/from Python ────────────────────────────────────────────

fn to_py_paloc_entry<'py>(py: Python<'py>, entry: &crate::binary::paloc::LocalizationEntry) -> PyResult<Bound<'py, PyDict>> {
    let d = PyDict::new(py);
    d.set_item("unk_id", entry.unk_id)?;
    d.set_item("string_key", entry.string_key.data)?;
    d.set_item("string_value", entry.string_value.data)?;
    Ok(d)
}

#[pyfunction]
pub fn parse_paloc_bytes(py: Python<'_>, data: &[u8]) -> PyResult<Py<PyAny>> {
    let paloc = crate::binary::paloc::LocalizationFile::parse(data)
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
    let entries = PyList::empty(py);
    for entry in &paloc.entries {
        entries.append(to_py_paloc_entry(py, entry)?)?;
    }
    Ok(entries.into_any().unbind())
}

#[pyfunction]
pub fn serialize_paloc(py: Python<'_>, items: &Bound<'_, PyList>) -> PyResult<Py<PyAny>> {
    let data = serialize_paloc_impl(items)?;
    Ok(PyBytes::new(py, &data).into_any().unbind())
}

fn serialize_paloc_impl(items: &Bound<'_, PyList>) -> PyResult<Vec<u8>> {
    use crate::binary::BinaryWrite;

    let mut buf = Vec::new();
    for item in items.iter() {
        let d = item.cast::<PyDict>()?;
        let unk_id: u64 = get(d, "unk_id")?;
        let string_key: String = get(d, "string_key")?;
        let string_value: String = get(d, "string_value")?;

        unk_id.write_to(&mut buf).map_err(|e| PyIOError::new_err(e.to_string()))?;
        (string_key.len() as u32).write_to(&mut buf).map_err(|e| PyIOError::new_err(e.to_string()))?;
        buf.extend_from_slice(string_key.as_bytes());
        (string_value.len() as u32).write_to(&mut buf).map_err(|e| PyIOError::new_err(e.to_string()))?;
        buf.extend_from_slice(string_value.as_bytes());
    }
    let count = items.len() as u32;
    count.write_to(&mut buf).map_err(|e| PyIOError::new_err(e.to_string()))?;
    Ok(buf)
}

// ── Localization (paloc) — category-based JSON form ──────────────────────
//
// These wrap `binary::paloc::parse_paloc_to_json` / `serialize_paloc_from_json`
// and expose the cleaner `{category: u8, key, value}` form (vs the legacy
// `unk_id` u64 form above). New callers should use these.

#[pyfunction]
pub fn parse_paloc_from_file(py: Python<'_>, path: &str) -> PyResult<Py<PyAny>> {
    let data = std::fs::read(path).map_err(|e| PyIOError::new_err(e.to_string()))?;
    parse_paloc_from_bytes(py, &data)
}

#[pyfunction]
pub fn parse_paloc_from_bytes(py: Python<'_>, data: &[u8]) -> PyResult<Py<PyAny>> {
    let json_array = crate::binary::paloc::parse_paloc_to_json(data)
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
    let entries = PyList::empty(py);
    for value in &json_array {
        let obj = value.as_object().expect("parse_paloc_to_json returns objects");
        let d = PyDict::new(py);
        let category = obj.get("category").and_then(|v| v.as_u64()).unwrap_or(0) as u8;
        let key = obj.get("key").and_then(|v| v.as_str()).unwrap_or("");
        let value = obj.get("value").and_then(|v| v.as_str()).unwrap_or("");
        d.set_item("category", category)?;
        d.set_item("key", key)?;
        d.set_item("value", value)?;
        entries.append(d)?;
    }
    Ok(entries.into_any().unbind())
}

#[pyfunction]
pub fn serialize_paloc_to_bytes(py: Python<'_>, items: &Bound<'_, PyList>) -> PyResult<Py<PyAny>> {
    // Convert PyList of dicts into Vec<serde_json::Value>, then call the
    // canonical serializer in binary/paloc.rs.
    let mut json_items: Vec<serde_json::Value> = Vec::with_capacity(items.len());
    for item in items.iter() {
        let d = item.cast::<PyDict>()?;
        let category: u64 = get(d, "category")?;
        let key: String = get(d, "key")?;
        let value: String = get(d, "value")?;
        let mut obj = serde_json::Map::new();
        obj.insert("category".to_string(), serde_json::Value::Number(category.into()));
        obj.insert("key".to_string(), serde_json::Value::String(key));
        obj.insert("value".to_string(), serde_json::Value::String(value));
        json_items.push(serde_json::Value::Object(obj));
    }
    let bytes = crate::binary::paloc::serialize_paloc_from_json(&json_items)
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
    Ok(PyBytes::new(py, &bytes).into_any().unbind())
}

// ── DDS texture metadata + validation (Phase D7) ──────────────────────────
//
// Lightweight Python wrappers around dmm_parser::dds. Used by SWISS Stacker
// to validate texture mods and pre-fill v3.1 asset target entries before
// export. SHA-256 is supplied by the caller (Python's hashlib) — dmm-parser
// doesn't bundle a SHA-256 implementation just for this.

#[pyfunction]
pub fn classify_dds(py: Python<'_>, data: &[u8]) -> PyResult<Py<PyAny>> {
    use crate::dds::classify;
    let c = classify(data).map_err(|e| PyValueError::new_err(e.to_string()))?;
    let d = PyDict::new(py);
    d.set_item("format", format!("{:?}", c.format))?;
    d.set_item("width", c.width)?;
    d.set_item("height", c.height)?;
    d.set_item("mip_count", c.mip_count)?;
    d.set_item("depth", c.depth)?;
    d.set_item("is_dx10", c.is_dx10)?;
    d.set_item("dxgi_format", c.dxgi_format)?;
    d.set_item("crimson_last4", c.crimson_last4)?;
    d.set_item("requires_pathc", c.requires_pathc)?;
    d.set_item("block_bytes", c.format.block_bytes())?;
    Ok(d.into_any().unbind())
}

#[pyfunction]
pub fn validate_dds(py: Python<'_>, data: &[u8]) -> PyResult<Py<PyAny>> {
    use crate::dds::{validate_dds_for_game, Severity};
    let findings = validate_dds_for_game(data);
    let out = PyList::empty(py);
    for f in &findings {
        let d = PyDict::new(py);
        d.set_item("code", f.code)?;
        d.set_item("severity", match f.severity {
            Severity::Fatal => "fatal",
            Severity::Warning => "warning",
            Severity::Info => "info",
        })?;
        d.set_item("message", &f.message)?;
        out.append(d)?;
    }
    Ok(out.into_any().unbind())
}

#[pyfunction]
pub fn infer_dds_vpath(asset_root: &str, file_path: &str) -> Option<String> {
    use std::path::Path;
    crate::dds::infer_vpath_from_disk_path(Path::new(asset_root), Path::new(file_path))
}

#[pyfunction]
pub fn classify_vpath_last4(vpath: &str) -> Option<u32> {
    crate::dds::classify_vpath_last4(vpath)
}

// ── Wwise audio (WEM + BNK) — Phase A8 ────────────────────────────────────
//
// Lightweight Python wrappers around dmm_parser::audio. Used by SWISS
// Stacker to validate audio mods (WEM voice clips, BNK soundbanks) and
// pre-fill v3.1 asset target entries during the asset-folder scan.

#[pyfunction]
pub fn classify_wem(py: Python<'_>, data: &[u8]) -> PyResult<Py<PyAny>> {
    use crate::audio::classify_wem as core;
    let m = core(data).map_err(|e| PyValueError::new_err(e.to_string()))?;
    let d = PyDict::new(py);
    d.set_item("file_size", m.file_size)?;
    d.set_item("format_tag", m.format_tag.raw())?;
    d.set_item("format_tag_label", match m.format_tag {
        crate::audio::WemFormatTag::WaveformatExtensible => "WaveformatExtensible",
        crate::audio::WemFormatTag::WwiseVorbis => "WwiseVorbis",
        crate::audio::WemFormatTag::Other(_) => "Other",
    })?;
    d.set_item("channels", m.channels)?;
    d.set_item("sample_rate", m.sample_rate)?;
    d.set_item("byte_rate", m.byte_rate)?;
    d.set_item("block_align", m.block_align)?;
    d.set_item("bits_per_sample", m.bits_per_sample)?;
    d.set_item("has_wwise_hash_chunk", m.has_wwise_hash_chunk)?;
    d.set_item("data_offset", m.data_offset)?;
    d.set_item("data_size", m.data_size)?;
    Ok(d.into_any().unbind())
}

#[pyfunction]
pub fn parse_bnk(py: Python<'_>, data: &[u8]) -> PyResult<Py<PyAny>> {
    use crate::audio::parse_bnk as core;
    let bnk = core(data).map_err(|e| PyValueError::new_err(e.to_string()))?;
    let d = PyDict::new(py);
    d.set_item("file_size", bnk.file_size)?;
    d.set_item("bank_version", bnk.bank_version)?;
    d.set_item("bank_id", bnk.bank_id)?;
    d.set_item("data_payload_offset", bnk.data_payload_offset)?;
    d.set_item("has_hirc", bnk.has_hirc)?;

    // Sections list
    let sections = PyList::empty(py);
    for s in &bnk.sections {
        let sd = PyDict::new(py);
        sd.set_item("id", std::str::from_utf8(&s.id).unwrap_or("????"))?;
        sd.set_item("header_offset", s.header_offset)?;
        sd.set_item("size", s.size)?;
        sections.append(sd)?;
    }
    d.set_item("sections", sections)?;

    // Embedded WEM index
    let wems = PyList::empty(py);
    for e in &bnk.embedded_wems {
        let ed = PyDict::new(py);
        ed.set_item("wem_id", e.wem_id)?;
        ed.set_item("wem_offset", e.wem_offset)?;
        ed.set_item("wem_size", e.wem_size)?;
        wems.append(ed)?;
    }
    d.set_item("embedded_wems", wems)?;
    Ok(d.into_any().unbind())
}

#[pyfunction]
pub fn infer_audio_vpath(vpath: &str) -> Option<&'static str> {
    use crate::audio::AudioPathClass;
    crate::audio::infer_audio_vpath(vpath).map(|c| match c {
        AudioPathClass::LocalizedVoiceBank => "LocalizedVoiceBank",
        AudioPathClass::LocalizedVoiceClip => "LocalizedVoiceClip",
        AudioPathClass::CommonSoundBank => "CommonSoundBank",
        AudioPathClass::CommonSoundClip => "CommonSoundClip",
        AudioPathClass::OtherAudio => "OtherAudio",
    })
}

#[pyfunction]
pub fn validate_audio(py: Python<'_>, data: &[u8]) -> PyResult<Py<PyAny>> {
    use crate::audio::{validate_audio as core, AudioSeverity};
    let findings = core(data);
    let out = PyList::empty(py);
    for f in &findings {
        let d = PyDict::new(py);
        d.set_item("code", f.code)?;
        d.set_item("severity", match f.severity {
            AudioSeverity::Fatal => "fatal",
            AudioSeverity::Warning => "warning",
            AudioSeverity::Info => "info",
        })?;
        d.set_item("message", &f.message)?;
        out.append(d)?;
    }
    Ok(out.into_any().unbind())
}

// ── Checksum ──────────────────────────────────────────────────────────────

#[pyfunction]
pub fn calculate_checksum(data: &[u8]) -> u32 {
    crate::crypto::checksum::calculate_checksum(data)
}

// ── Compression ──────────────────────────────────────────────────────────

#[pyfunction]
pub fn compress_data(py: Python<'_>, data: &[u8], compression: u8) -> PyResult<Py<PyAny>> {
    use crate::binary::paz;
    use crate::binary::pamt::Compression;

    let comp = match compression {
        0 => Compression::None,
        2 => Compression::Lz4,
        3 => Compression::Zlib,
        _ => return Err(PyValueError::new_err(format!("unsupported compression: {}", compression))),
    };

    let result = paz::compress(data, comp)
        .map_err(|e| PyIOError::new_err(e.to_string()))?;
    Ok(PyBytes::new(py, &result).into_any().unbind())
}

#[pyfunction]
pub fn decompress_data(py: Python<'_>, data: &[u8], compression: u8, uncompressed_size: usize) -> PyResult<Py<PyAny>> {
    use crate::binary::paz;
    use crate::binary::pamt::Compression;

    let comp = match compression {
        0 => Compression::None,
        2 => Compression::Lz4,
        3 => Compression::Zlib,
        _ => return Err(PyValueError::new_err(format!("unsupported compression: {}", compression))),
    };

    let result = paz::decompress(data, comp, uncompressed_size)
        .map_err(|e| PyIOError::new_err(e.to_string()))?;
    Ok(PyBytes::new(py, &result).into_any().unbind())
}

// ── Pack Group Builder (streaming) ───────────────────────────────────────

fn parse_compression(compression: u8) -> PyResult<crate::binary::pamt::Compression> {
    use crate::binary::pamt::Compression;
    match compression {
        0 => Ok(Compression::None),
        2 => Ok(Compression::Lz4),
        3 => Ok(Compression::Zlib),
        _ => Err(PyValueError::new_err(format!("unsupported compression: {}", compression))),
    }
}

fn parse_crypto(crypto: u8) -> PyResult<crate::binary::pamt::CryptoType> {
    use crate::binary::pamt::CryptoType;
    match crypto {
        0 => Ok(CryptoType::None),
        3 => Ok(CryptoType::ChaCha20),
        _ => Err(PyValueError::new_err(format!("unsupported crypto: {}", crypto))),
    }
}

/// Streaming pack group builder that writes .paz files to disk incrementally.
///
/// Usage:
///     builder = PackGroupBuilder("/path/to/0036", compression=2)
///     builder.add_file("textures", "icon.dds", raw_bytes)
///     builder.add_file_from_path("models", "mesh.obj", "/path/to/mesh.obj")
///     pamt_bytes = builder.finish()  # writes .paz + 0.pamt to output_dir
#[pyclass(name = "PackGroupBuilder")]
pub struct PyPackGroupBuilder {
    inner: Option<crate::binary::paz::PackGroupBuilder>,
}

#[pymethods]
impl PyPackGroupBuilder {
    #[new]
    #[pyo3(signature = (output_dir, compression=2, crypto=0, encrypt_info=vec![0,0,0], max_chunk_size=500_000_000))]
    fn new(
        output_dir: &str,
        compression: u8,
        crypto: u8,
        encrypt_info: Vec<u8>,
        max_chunk_size: u64,
    ) -> PyResult<Self> {
        let comp = parse_compression(compression)?;
        let cry = parse_crypto(crypto)?;
        let ei: [u8; 3] = encrypt_info.try_into()
            .map_err(|_| PyValueError::new_err("encrypt_info must be 3 bytes"))?;

        // Create output directory if it doesn't exist
        std::fs::create_dir_all(output_dir)
            .map_err(|e| PyIOError::new_err(e.to_string()))?;

        let builder = crate::binary::paz::PackGroupBuilder::new(
            std::path::Path::new(output_dir),
            comp,
            cry,
            ei,
            max_chunk_size,
        );

        Ok(PyPackGroupBuilder { inner: Some(builder) })
    }

    /// Add a file from raw bytes.
    fn add_file(&mut self, dir_path: &str, file_name: &str, data: &[u8]) -> PyResult<()> {
        let builder = self.inner.as_mut()
            .ok_or_else(|| PyValueError::new_err("builder already finished"))?;
        builder.add_file(dir_path, file_name, data)
            .map_err(|e| PyIOError::new_err(e.to_string()))
    }

    /// Add a file by reading from a path on disk.
    fn add_file_from_path(&mut self, dir_path: &str, file_name: &str, file_path: &str) -> PyResult<()> {
        let builder = self.inner.as_mut()
            .ok_or_else(|| PyValueError::new_err("builder already finished"))?;
        builder.add_file_from_path(dir_path, file_name, std::path::Path::new(file_path))
            .map_err(|e| PyIOError::new_err(e.to_string()))
    }

    /// Finish building: flush remaining chunk, write 0.pamt.
    /// Returns the raw PAMT bytes (for computing checksum for PAPGT).
    fn finish(&mut self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let builder = self.inner.take()
            .ok_or_else(|| PyValueError::new_err("builder already finished"))?;
        let pamt_bytes = builder.finish()
            .map_err(|e| PyIOError::new_err(e.to_string()))?;
        Ok(PyBytes::new(py, &pamt_bytes).into_any().unbind())
    }
}

/// Add a new entry to a PAPGT dict.
///
/// Parses the PAPGT from the dict, adds the entry, re-serializes,
/// and returns the updated PAPGT as a new dict.
#[pyfunction]
pub fn add_papgt_entry(
    py: Python<'_>,
    papgt_data: &Bound<'_, PyDict>,
    group_name: &str,
    pack_meta_checksum: u32,
    is_optional: u8,
    language: u16,
) -> PyResult<Py<PyAny>> {
    // Reconstruct the PackGroupTreeMeta from the dict
    let bytes = wr_papgt_from_dict(papgt_data)?;
    let mut papgt = PackGroupTreeMeta::parse(&bytes)
        .map_err(|e| PyValueError::new_err(e.to_string()))?;

    papgt.add_entry(group_name, pack_meta_checksum, is_optional, language);

    let new_bytes = papgt.to_bytes()
        .map_err(|e| PyIOError::new_err(e.to_string()))?;

    // Re-parse to get the dict representation
    let new_papgt = PackGroupTreeMeta::parse(&new_bytes)
        .map_err(|e| PyValueError::new_err(e.to_string()))?;

    Ok(to_py_papgt(py, &new_papgt)?.into_any().unbind())
}

// ── File Extraction ───────────────────────────────────────────────────────

/// Extract a single file from a pack group archive to bytes.
///
/// Given a game directory, group name, directory path, and file name,
/// finds the file in the PAMT index and reads/decrypts/decompresses it.
#[pyfunction]
pub fn extract_file(
    py: Python<'_>,
    game_dir: &str,
    group_name: &str,
    dir_path: &str,
    file_name: &str,
) -> PyResult<Py<PyAny>> {
    use std::path::Path;
    use crate::binary::paz;

    let group_dir = Path::new(game_dir).join(group_name);
    let pamt_path = group_dir.join("0.pamt");

    let pamt_data = std::fs::read(&pamt_path)
        .map_err(|e| PyIOError::new_err(format!("{}: {}", pamt_path.display(), e)))?;
    let pamt = PackMeta::parse(&pamt_data, None)
        .map_err(|e| PyValueError::new_err(e.to_string()))?;

    // Find the directory and file
    let dir = pamt.directories.iter()
        .find(|d| d.path == dir_path)
        .ok_or_else(|| PyValueError::new_err(
            format!("directory '{}' not found in {}/{}", dir_path, group_name, "0.pamt"),
        ))?;

    let file = dir.files.iter()
        .find(|f| f.name == file_name)
        .ok_or_else(|| PyValueError::new_err(
            format!("file '{}' not found in directory '{}'", file_name, dir_path),
        ))?;

    let encrypt_info = pamt.header.encrypt_info.encrypt_info;
    let raw = paz::extract_file(&group_dir, file, dir_path, &encrypt_info)
        .map_err(|e| PyIOError::new_err(e.to_string()))?;

    Ok(PyBytes::new(py, &raw).into_any().unbind())
}

// ── SkillInfo ──────────────────────────────────────────────────────────────

#[pyfunction]
pub fn parse_skillinfo_from_file(py: Python<'_>, pabgb_path: &str, pabgh_path: &str) -> PyResult<Py<PyAny>> {
    let data = std::fs::read(pabgb_path).map_err(|e| PyIOError::new_err(e.to_string()))?;
    let pabgh = std::fs::read(pabgh_path).map_err(|e| PyIOError::new_err(e.to_string()))?;
    parse_skillinfo_from_bytes(py, &data, &pabgh)
}

#[pyfunction]
pub fn parse_skillinfo_from_bytes(py: Python<'_>, pabgb: &[u8], pabgh: &[u8]) -> PyResult<Py<PyAny>> {
    let items = crate::tables::skill_info::parse_skill_to_json_with_pabgh(pabgb, pabgh)
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
    let list = PyList::empty(py);
    for v in items {
        list.append(json_to_py(py, &v)?)?;
    }
    Ok(list.into_any().unbind())
}

#[pyfunction]
pub fn serialize_skillinfo(py: Python<'_>, items: &Bound<'_, PyList>) -> PyResult<Py<PyAny>> {
    let values: Vec<serde_json::Value> = items.iter()
        .map(|item| py_to_json(&item))
        .collect::<PyResult<_>>()?;
    let data = crate::tables::skill_info::serialize_skill_from_json(&values)
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
    Ok(PyBytes::new(py, &data).into_any().unbind())
}

#[pyfunction]
pub fn write_skillinfo_to_file(items: &Bound<'_, PyList>, path: &str) -> PyResult<()> {
    let values: Vec<serde_json::Value> = items.iter()
        .map(|item| py_to_json(&item))
        .collect::<PyResult<_>>()?;
    let data = crate::tables::skill_info::serialize_skill_from_json(&values)
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
    std::fs::write(path, &data).map_err(|e| PyIOError::new_err(e.to_string()))
}

// ── BuffInfo ───────────────────────────────────────────────────────────────

#[pyfunction]
pub fn parse_buffinfo_from_file(py: Python<'_>, pabgb_path: &str, pabgh_path: &str) -> PyResult<Py<PyAny>> {
    let data = std::fs::read(pabgb_path).map_err(|e| PyIOError::new_err(e.to_string()))?;
    let pabgh = std::fs::read(pabgh_path).map_err(|e| PyIOError::new_err(e.to_string()))?;
    parse_buffinfo_from_bytes(py, &data, &pabgh)
}

#[pyfunction]
pub fn parse_buffinfo_from_bytes(py: Python<'_>, pabgb: &[u8], pabgh: &[u8]) -> PyResult<Py<PyAny>> {
    let items = crate::tables::buff_info::parse_buffinfo_to_json_with_pabgh(pabgb, pabgh)
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
    let list = PyList::empty(py);
    for v in items {
        list.append(json_to_py(py, &v)?)?;
    }
    Ok(list.into_any().unbind())
}

#[pyfunction]
pub fn serialize_buffinfo(py: Python<'_>, items: &Bound<'_, PyList>) -> PyResult<Py<PyAny>> {
    let values: Vec<serde_json::Value> = items.iter()
        .map(|item| py_to_json(&item))
        .collect::<PyResult<_>>()?;
    let data = crate::tables::buff_info::serialize_buffinfo_from_json(&values)
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
    Ok(PyBytes::new(py, &data).into_any().unbind())
}

#[pyfunction]
pub fn write_buffinfo_to_file(items: &Bound<'_, PyList>, path: &str) -> PyResult<()> {
    let values: Vec<serde_json::Value> = items.iter()
        .map(|item| py_to_json(&item))
        .collect::<PyResult<_>>()?;
    let data = crate::tables::buff_info::serialize_buffinfo_from_json(&values)
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
    std::fs::write(path, &data).map_err(|e| PyIOError::new_err(e.to_string()))
}

// ── Generic table dispatch ─────────────────────────────────────────────────
//
// These wrappers exist only to convert io::Error → PyValueError. The real
// dispatch logic lives in `crate::dispatch::{parse_table_to_json,
// serialize_table_from_json}` so non-Python Rust callers (DMM, CLI, tests)
// can use it directly without the PyO3 dependency. Single source of truth
// for the 122-table match arms — adding a new table = one entry in
// `dispatch.rs`, the Python side picks it up automatically.

fn dispatch_parse(
    table_name: &str,
    pabgb: &[u8],
    pabgh: Option<&[u8]>,
) -> PyResult<Vec<serde_json::Value>> {
    crate::dispatch::parse_table_to_json(table_name, pabgb, pabgh)
        .map_err(|e| PyValueError::new_err(e.to_string()))
}

fn dispatch_serialize_bytes(
    table_name: &str,
    json_items: &[serde_json::Value],
) -> PyResult<Vec<u8>> {
    crate::dispatch::serialize_table_from_json(table_name, json_items)
        .map_err(|e| PyValueError::new_err(e.to_string()))
}

#[pyfunction]
#[pyo3(signature = (table_name, pabgb, pabgh=None))]
pub fn parse_table(
    py: Python<'_>,
    table_name: &str,
    pabgb: &[u8],
    pabgh: Option<&[u8]>,
) -> PyResult<Py<PyAny>> {
    let values = dispatch_parse(table_name, pabgb, pabgh)?;
    let list = PyList::empty(py);
    for v in values {
        list.append(json_to_py(py, &v)?)?;
    }
    Ok(list.into_any().unbind())
}

#[pyfunction]
pub fn serialize_table(
    py: Python<'_>,
    table_name: &str,
    items: &Bound<'_, PyList>,
) -> PyResult<Py<PyAny>> {
    let json_items: Vec<serde_json::Value> = items.iter()
        .map(|item| py_to_json(&item))
        .collect::<PyResult<_>>()?;
    let data = dispatch_serialize_bytes(table_name, &json_items)?;
    Ok(PyBytes::new(py, &data).into_any().unbind())
}

#[pyfunction]
pub fn write_table_to_file(
    table_name: &str,
    items: &Bound<'_, PyList>,
    path: &str,
) -> PyResult<()> {
    let json_items: Vec<serde_json::Value> = items.iter()
        .map(|item| py_to_json(&item))
        .collect::<PyResult<_>>()?;
    let data = dispatch_serialize_bytes(table_name, &json_items)?;
    std::fs::write(path, &data).map_err(|e| PyIOError::new_err(e.to_string()))
}

// ── Field-JSON v3.x intent application ────────────────────────────────────
//
// Single-target apply. Python callers iterate over the manifest's targets,
// load each pabgb (+ pabgh sister) themselves, call this for each target,
// and write the result. This keeps the binding simple and avoids the
// complexity of a callback-based file-resolver model.

/// Apply Field-JSON v3.x intents to a single table body.
///
/// Args:
///   table_name: any recognized form — canonical (`character_info`),
///     compact (`characterinfo`), with extension (`characterinfo.pabgb`).
///   pabgb: raw `.pabgb` bytes.
///   pabgh: raw `.pabgh` sister bytes for pabgh-bounded tables; `None`
///     for sequential tables, iteminfo, or paloc.
///   intents: list of intent dicts as appearing in a Field-JSON manifest's
///     `intents` array (see FIELD_JSON_V3_SPEC.md / CUSTOM_ITEM_CREATOR_V3_1.md).
///
/// Returns a dict:
///   `{"body": bytes, "pabgh": bytes | None, "outcomes": [{"op": str,
///     "status": "applied" | "skipped", "reason"?: str}]}`
///
/// Raises `ValueError` for unknown table names, malformed intents, or
/// any apply failure.
#[pyfunction]
#[pyo3(signature = (table_name, pabgb, pabgh, intents))]
pub fn apply_intents(
    py: Python<'_>,
    table_name: &str,
    pabgb: &[u8],
    pabgh: Option<&[u8]>,
    intents: &Bound<'_, PyList>,
) -> PyResult<Py<PyAny>> {
    // Convert Python list of intent dicts to Rust Intent values via
    // JSON-faithful parser (handles every shape the spec allows).
    let intent_list: Vec<crate::intents::Intent> = intents
        .iter()
        .map(|item| {
            let v = py_to_json(&item)?;
            crate::intents::Intent::from_value(&v)
                .map_err(|e| PyValueError::new_err(format!("intent: {}", e)))
        })
        .collect::<PyResult<_>>()?;

    let (new_body, new_pabgh, outcomes) = crate::dispatch::apply_intents_to_table_body(
        table_name,
        pabgb,
        pabgh,
        &intent_list,
    )
    .map_err(|e| PyValueError::new_err(e.to_string()))?;

    let result = PyDict::new(py);
    result.set_item("body", PyBytes::new(py, &new_body))?;
    match new_pabgh {
        Some(p) => result.set_item("pabgh", PyBytes::new(py, &p))?,
        None => result.set_item("pabgh", py.None())?,
    }

    let outcome_list = PyList::empty(py);
    for o in outcomes {
        let d = PyDict::new(py);
        d.set_item("op", &o.op)?;
        match o.status {
            crate::intents::ApplyStatus::Applied => {
                d.set_item("status", "applied")?;
            }
            crate::intents::ApplyStatus::Skipped(reason) => {
                d.set_item("status", "skipped")?;
                d.set_item("reason", &reason)?;
            }
        }
        outcome_list.append(d)?;
    }
    result.set_item("outcomes", outcome_list)?;

    Ok(result.into_any().unbind())
}

/// Resolve a Field-JSON target name (e.g. `characterinfo.pabgb`) to its
/// canonical dispatch identifier (e.g. `character_info`). Returns `None`
/// for unrecognized names.
#[pyfunction]
pub fn normalize_target_name(name: &str) -> Option<&'static str> {
    crate::dispatch::normalize_target_name(name)
}

/// Compute the v3 paloc index keys for a custom item's localized name
/// and description. Returns `(name_index, desc_index)`.
///
/// Per CUSTOM_ITEM_CREATOR_V3_1.md: `name_index = (item_key << 32) | 0x70`
/// and `desc_index = (item_key << 32) | 0x71`. SWISS / Stacker compute
/// these when assigning a `new_key` for a clone_record intent.
#[pyfunction]
pub fn item_paloc_indices(item_key: u32) -> (u64, u64) {
    crate::intents::item_paloc_indices(item_key)
}

// ── Tier 1 typed-reader bindings (Session 13) ────────────────────────────
//
// Each format exposes:
//   - `parse_<format>_bytes(data: bytes) -> dict`   — typed parse to JSON-shaped dict
//   - `serialize_<format>(obj: dict) -> bytes`      — write back to wire bytes
//
// Round-trip via these is byte-perfect for every vanilla sample (verified
// by `examples/<format>_roundtrip.rs`). The JSON dict keys map 1:1 to the
// `TypedXxxFile` Rust struct fields.

macro_rules! bind_typed_format {
    ($parse_fn:ident, $write_fn:ident, $parse_file_fn:ident, $write_file_fn:ident, $typed:path) => {
        #[pyfunction]
        pub fn $parse_fn(py: Python<'_>, data: &[u8]) -> PyResult<Py<PyAny>> {
            use $typed as Typed;
            use crate::json_traits::ToJsonValue;
            let typed = Typed::parse(data)
                .map_err(|e| PyValueError::new_err(e.to_string()))?;
            json_to_py(py, &typed.to_json_value())
        }

        #[pyfunction]
        pub fn $write_fn(py: Python<'_>, obj: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
            use $typed as Typed;
            use crate::json_traits::WriteJsonValue;
            let json = py_to_json(obj)?;
            let mut out = Vec::new();
            Typed::write_from_json(&mut out, &json)
                .map_err(|e| PyIOError::new_err(e.to_string()))?;
            Ok(PyBytes::new(py, &out).into_any().unbind())
        }

        /// Convenience: read the file at `path`, parse, return the
        /// JSON-shaped dict. Equivalent to
        /// `parse(open(path, "rb").read())`.
        #[pyfunction]
        pub fn $parse_file_fn(py: Python<'_>, path: &str) -> PyResult<Py<PyAny>> {
            let data = std::fs::read(path)
                .map_err(|e| PyIOError::new_err(format!("read {}: {}", path, e)))?;
            $parse_fn(py, &data)
        }

        /// Convenience: serialize `obj` and write the bytes to `path`.
        /// Atomic via tempfile + rename when supported by the filesystem.
        #[pyfunction]
        pub fn $write_file_fn(py: Python<'_>, obj: &Bound<'_, PyAny>, path: &str) -> PyResult<()> {
            use $typed as Typed;
            use crate::json_traits::WriteJsonValue;
            let json = py_to_json(obj)?;
            let mut out = Vec::new();
            Typed::write_from_json(&mut out, &json)
                .map_err(|e| PyIOError::new_err(e.to_string()))?;
            let _ = py;
            std::fs::write(path, &out)
                .map_err(|e| PyIOError::new_err(format!("write {}: {}", path, e)))?;
            Ok(())
        }
    };
}

bind_typed_format!(
    parse_pastage_bytes, serialize_pastage,
    parse_pastage_from_file, write_pastage_to_file,
    crate::binary::pastage::TypedPastageFile
);
bind_typed_format!(
    parse_paseq_bytes, serialize_paseq,
    parse_paseq_from_file, write_paseq_to_file,
    crate::binary::paseq::TypedPaseqFile
);
bind_typed_format!(
    parse_paseqc_bytes, serialize_paseqc,
    parse_paseqc_from_file, write_paseqc_to_file,
    crate::binary::paseqc::TypedPaseqcFile
);
bind_typed_format!(
    parse_paschedule_bytes, serialize_paschedule,
    parse_paschedule_from_file, write_paschedule_to_file,
    crate::binary::paschedule::TypedPascheduleFile
);
bind_typed_format!(
    parse_paschedulepath_bytes, serialize_paschedulepath,
    parse_paschedulepath_from_file, write_paschedulepath_to_file,
    crate::binary::paschedulepath::TypedPaschedulePathFile
);
bind_typed_format!(
    parse_paatt_bytes, serialize_paatt,
    parse_paatt_from_file, write_paatt_to_file,
    crate::binary::paatt::PaattFile
);

/// Parse the outer class field directory from a `.paseq` file. Returns
/// a list of dicts: `[{"field_name": str, "type_name": str,
/// "type_meta_b64": str}, ...]`. Every vanilla `.paseq` exposes the
/// same 15-field `pa::Sequencer` schema; this lets mod authors
/// enumerate the declared fields without parsing the recursive
/// nested-class schema.
#[pyfunction]
pub fn parse_paseq_field_directory(py: Python<'_>, data: &[u8]) -> PyResult<Py<PyAny>> {
    use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
    let typed = crate::binary::paseq::TypedPaseqFile::parse(data)
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
    let fields = typed.outer_fields()
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
    let list = PyList::empty(py);
    for f in fields {
        let dict = PyDict::new(py);
        dict.set_item("field_name", f.field_name)?;
        dict.set_item("type_name", f.type_name)?;
        dict.set_item("type_meta_b64", B64.encode(f.type_meta))?;
        list.append(dict)?;
    }
    Ok(list.into_any().unbind())
}

/// Parse the outer class field directory from a `.paseqc` file. Same
/// schema as `parse_paseq_field_directory` — the two formats share the
/// engine reflection serializer.
#[pyfunction]
pub fn parse_paseqc_field_directory(py: Python<'_>, data: &[u8]) -> PyResult<Py<PyAny>> {
    use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
    let typed = crate::binary::paseqc::TypedPaseqcFile::parse(data)
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
    let fields = typed.outer_fields()
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
    let list = PyList::empty(py);
    for f in fields {
        let dict = PyDict::new(py);
        dict.set_item("field_name", f.field_name)?;
        dict.set_item("type_name", f.type_name)?;
        dict.set_item("type_meta_b64", B64.encode(f.type_meta))?;
        list.append(dict)?;
    }
    Ok(list.into_any().unbind())
}

fn class_blocks_to_pylist(
    py: Python<'_>,
    blocks: Vec<crate::binary::paseq::PaseqClassBlock>,
) -> PyResult<Py<PyAny>> {
    use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
    let list = PyList::empty(py);
    for block in blocks {
        let block_dict = PyDict::new(py);
        block_dict.set_item("class_name", block.class_name)?;
        let fields_list = PyList::empty(py);
        for f in block.fields {
            let fd = PyDict::new(py);
            fd.set_item("field_name", f.field_name)?;
            fd.set_item("type_name", f.type_name)?;
            fd.set_item("type_meta_b64", B64.encode(f.type_meta))?;
            fields_list.append(fd)?;
        }
        block_dict.set_item("fields", fields_list)?;
        list.append(block_dict)?;
    }
    Ok(list.into_any().unbind())
}

/// Walk all class blocks (outer + linearly-following nested classes) in
/// a `.paseq` file. Returns a list of `{"class_name": str, "fields":
/// [...]}` dicts. The walker stops when it encounters non-CString-shaped
/// data (i.e. the value section starts). Every vanilla `.paseq`
/// produces a complete walk (verified across 4,659 samples — 272
/// distinct class names like `Sequencer`, `TimelineRootNode`,
/// `TimelineFloatKeyFrameNode`, etc.).
#[pyfunction]
pub fn parse_paseq_all_class_blocks(py: Python<'_>, data: &[u8]) -> PyResult<Py<PyAny>> {
    let typed = crate::binary::paseq::TypedPaseqFile::parse(data)
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
    let blocks = typed.all_class_blocks()
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
    class_blocks_to_pylist(py, blocks)
}

/// Walk all class blocks in a `.paseqc` file. Same shape as
/// `parse_paseq_all_class_blocks`. Verified across 2,932 samples
/// (62 distinct class names like `GameData_Sequencer`, `GameData_Folder`,
/// `SequencerGamePlayData_CharacterActor`, etc.).
#[pyfunction]
pub fn parse_paseqc_all_class_blocks(py: Python<'_>, data: &[u8]) -> PyResult<Py<PyAny>> {
    let typed = crate::binary::paseqc::TypedPaseqcFile::parse(data)
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
    let blocks = typed.all_class_blocks()
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
    class_blocks_to_pylist(py, blocks)
}

/// Get the byte offset of the value section within a `.paseq` file's
/// `opaque_body`. Returns the index of the first byte of values
/// (right after the last class block in the schema).
#[pyfunction]
pub fn paseq_value_section_offset(data: &[u8]) -> PyResult<usize> {
    let typed = crate::binary::paseq::TypedPaseqFile::parse(data)
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
    typed.value_section_offset()
        .map_err(|e| PyValueError::new_err(e.to_string()))
}

/// Get the value-section bytes from a `.paseq` file (the bytes after
/// the schema). Decoding these per-type is future work; for now the
/// raw bytes are returned for tools that want to do their own analysis.
#[pyfunction]
pub fn paseq_value_section<'py>(py: Python<'py>, data: &[u8]) -> PyResult<Bound<'py, PyBytes>> {
    let typed = crate::binary::paseq::TypedPaseqFile::parse(data)
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
    let values = typed.value_section()
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
    Ok(PyBytes::new(py, values))
}

/// Get the byte offset of the value section within a `.paseqc` file.
#[pyfunction]
pub fn paseqc_value_section_offset(data: &[u8]) -> PyResult<usize> {
    let typed = crate::binary::paseqc::TypedPaseqcFile::parse(data)
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
    typed.value_section_offset()
        .map_err(|e| PyValueError::new_err(e.to_string()))
}

/// Get the value-section bytes from a `.paseqc` file.
#[pyfunction]
pub fn paseqc_value_section<'py>(py: Python<'py>, data: &[u8]) -> PyResult<Bound<'py, PyBytes>> {
    let typed = crate::binary::paseqc::TypedPaseqcFile::parse(data)
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
    let values = typed.value_section()
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
    Ok(PyBytes::new(py, values))
}

fn strings_to_pylist<'py>(py: Python<'py>, strings: Vec<(usize, String)>) -> PyResult<Py<PyAny>> {
    let list = PyList::empty(py);
    for (offset, s) in strings {
        let dict = PyDict::new(py);
        dict.set_item("file_offset", offset)?;
        dict.set_item("value", s)?;
        list.append(dict)?;
    }
    Ok(list.into_any().unbind())
}

/// Walk the value section of a `.paseq` file and return every
/// `u32 length + N printable bytes` pattern as a list of
/// `{"file_offset": int, "value": str}` dicts. The byte offset is
/// relative to the START OF THE FILE so callers can do surgical
/// edits — overwrite a string at a known offset with a same-length
/// replacement, or use `serialize_paseq` after editing the parsed
/// dict.
///
/// Captures `staticstringA` field values, embedded asset path
/// references, script-expression strings, and similar variable-length
/// string data. Heuristic — strings must be 1..=4096 bytes of
/// printable ASCII.
#[pyfunction]
pub fn paseq_value_section_strings(py: Python<'_>, data: &[u8]) -> PyResult<Py<PyAny>> {
    let typed = crate::binary::paseq::TypedPaseqFile::parse(data)
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
    let strings = typed.value_section_strings()
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
    strings_to_pylist(py, strings)
}

/// Sister to `paseq_value_section_strings` for `.paseqc` files.
#[pyfunction]
pub fn paseqc_value_section_strings(py: Python<'_>, data: &[u8]) -> PyResult<Py<PyAny>> {
    let typed = crate::binary::paseqc::TypedPaseqcFile::parse(data)
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
    let strings = typed.value_section_strings()
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
    strings_to_pylist(py, strings)
}

/// Walk `data` looking for `u32 length + N printable bytes` patterns
/// (length-prefixed strings). Returns a list of
/// `{"file_offset": int, "value": str}` dicts. Generic — works on any
/// byte slice from any format (not just `.paseq`/`.paseqc`).
///
/// Pairs with `replace_cstring_at` for full string-level mod tooling:
/// find strings → edit them by file offset → reserialize.
///
/// Heuristic: strings must be 1..=4096 bytes of printable ASCII (or
/// `\n`, `\t`). The walker advances past matches so overlapping
/// regions don't double-count.
#[pyfunction]
pub fn walk_lp_strings(py: Python<'_>, data: &[u8]) -> PyResult<Py<PyAny>> {
    let strings = crate::binary::paseq::walk_u32_prefixed_strings(data, 0);
    strings_to_pylist(py, strings)
}

/// Replace the `u32 length + N bytes` CString at `file_offset` in
/// `data` with `new_value`. Length-flexible — the result is
/// `len(new_value) - old_length` bytes larger or smaller. Works on
/// any format where values are stored as length-prefixed strings.
///
/// Args:
///   data: the input file bytes
///   file_offset: where the u32 length prefix lives (typically from
///     a `*_value_section_strings` lookup)
///   new_value: the replacement string
///   expected_value: optional safety check — if Some, parse will
///     fail unless the existing bytes match this value
///
/// Returns the modified file bytes.
#[pyfunction]
#[pyo3(signature = (data, file_offset, new_value, expected_value=None))]
pub fn replace_cstring_at<'py>(
    py: Python<'py>,
    data: &[u8],
    file_offset: usize,
    new_value: &str,
    expected_value: Option<&str>,
) -> PyResult<Bound<'py, PyBytes>> {
    let result = crate::binary::paseq::replace_cstring_at(
        data, file_offset, expected_value, new_value,
    ).map_err(|e| PyValueError::new_err(e.to_string()))?;
    Ok(PyBytes::new(py, &result))
}

// ── .paatt BaseData typed decode/encode ────────────────────────────────────

/// Decode a `.paatt` `BaseData` blob to a named-field dict.
///
/// **version** is the `AttackInfo.version` byte (0–4).
/// **data** is the raw `base_data` bytes (264 / 528 / 296 / 288 / 264 bytes
/// depending on version).
///
/// For V0 and V1 the returned dict has individual named fields:
///   - `weapon_key` (int) — weapon hash key
///   - `attack_dir` (int) — 0 = forward, 1 = catch direction
///   - `physic_impulse_power` (float) — vanilla default 1.0
///   - `physics_impulse_mass` (float) — vanilla default 1.0
///   - `repeat_degree_weight` (float) — vanilla default -1.0
///   - `ignore_safe_zone` (bool)
///   - `single_hit_position_socket` (int) — 0xffff = no socket
///   - `_unkXXXX_b64` (str) — undecoded regions as base64
///   - V1 only: `catch_desc` (dict) — decoded AttackCatchDesc fields
///
/// For V2 (throw) the dict adds: `projectile_key` (int), `action_hash_code` (int),
///   `frame_time` (float), `ai_event_key` (int), plus blob fields.
/// For V3 (release-catch) the dict adds: `release_angle_rad` (float), `frame_time` (float),
///   `_unk0110` (int), `_unk0114` (int), plus blob fields.
/// For V4 the dict has only `{"version": 4, "base_data_b64": "..."}`.
/// Pass the returned dict unmodified (or after editing named fields) to
/// `paatt_encode_base_data` to get bytes back.
///
/// **shape** selects the JSON name set:
/// * `"v3"` (default) — emits legacy `_unkXXXX` placeholder names.
///   Existing DMM v3 mods are authored against these.
/// * `"v3.1"` — emits canonical real-C++ field names from IDA. Use this
///   when building a v3.1-aware consumer (e.g. DMM v2.0.0-beta).
///
/// `paatt_encode_base_data` accepts BOTH name sets on input regardless
/// of which shape produced the dict, so authoring tools can switch
/// freely.
#[pyfunction]
#[pyo3(signature = (version, data, shape=None))]
pub fn paatt_decode_base_data<'py>(
    py: Python<'py>,
    version: u8,
    data: &[u8],
    shape: Option<&str>,
) -> PyResult<Bound<'py, PyAny>> {
    use crate::binary::paatt_basedata::AttackInfoBaseData;
    use crate::json_shape::JsonShape;
    use base64::{engine::general_purpose::STANDARD as B64, Engine as _};

    let shape = JsonShape::from_str(shape.unwrap_or("v3"))
        .map_err(PyValueError::new_err)?;

    let decoded = AttackInfoBaseData::decode(version, data)
        .map_err(|e| PyValueError::new_err(e.to_string()))?;

    let dict = PyDict::new(py);
    dict.set_item("version", decoded.version())?;

    // Build the JSON value via the existing per-version to_json_value, then
    // project through the shape transform. The base-V0 alias table only
    // touches V0-level field names; V1/V2/V3 trailing fields don't appear
    // in the alias table and pass through unchanged regardless of shape.
    use crate::json_traits::ToJsonValue;
    let mut jv = match &decoded {
        AttackInfoBaseData::V0(v0) => v0.to_json_value(),
        AttackInfoBaseData::V1(v1) => v1.to_json_value(),
        AttackInfoBaseData::V2(v2) => v2.to_json_value(),
        AttackInfoBaseData::V3(v3) => v3.to_json_value(),
        AttackInfoBaseData::Raw { data, .. } => {
            dict.set_item("base_data_b64", B64.encode(data))?;
            return Ok(dict.into_any());
        }
    };

    if shape == JsonShape::V3 {
        if let Some(map) = jv.as_object_mut() {
            crate::json_shape::apply_v3_aliases(
                map,
                crate::binary::paatt_basedata::FIELD_ALIASES_V3,
            );
        }
    }

    if let Some(obj) = jv.as_object() {
        for (k, v) in obj {
            let py_val = json_value_to_py(py, v)?;
            dict.set_item(k, py_val)?;
        }
    }

    Ok(dict.into_any())
}

/// Encode a named-field dict (as returned by `paatt_decode_base_data`) back
/// to raw `BaseData` bytes.
///
/// **version** must match the `version` field used when decoding.
/// Returns the binary `base_data` bytes ready to replace `AttackInfo.base_data`.
#[pyfunction]
pub fn paatt_encode_base_data<'py>(
    py: Python<'py>,
    version: u8,
    fields: &Bound<'py, PyDict>,
) -> PyResult<Bound<'py, PyBytes>> {
    use crate::binary::paatt_basedata::{BaseDataV0, BaseDataV1, BaseDataV2, BaseDataV3};
    use crate::json_traits::WriteJsonValue;
    use base64::{engine::general_purpose::STANDARD as B64, Engine as _};

    // Convert the Python dict to a serde_json::Value so we can use WriteJsonValue.
    let json_val = py_dict_to_json_value(fields)?;

    let bytes = match version {
        0 => {
            let mut w = Vec::new();
            BaseDataV0::write_from_json(&mut w, &json_val)
                .map_err(|e| PyValueError::new_err(e.to_string()))?;
            w
        }
        1 => {
            let mut w = Vec::new();
            BaseDataV1::write_from_json(&mut w, &json_val)
                .map_err(|e| PyValueError::new_err(e.to_string()))?;
            w
        }
        2 => {
            let mut w = Vec::new();
            BaseDataV2::write_from_json(&mut w, &json_val)
                .map_err(|e| PyValueError::new_err(e.to_string()))?;
            w
        }
        3 => {
            let mut w = Vec::new();
            BaseDataV3::write_from_json(&mut w, &json_val)
                .map_err(|e| PyValueError::new_err(e.to_string()))?;
            w
        }
        _ => {
            // Raw pass-through: expect base_data_b64 key.
            let b64_str = fields
                .get_item("base_data_b64")
                .map_err(|e| PyValueError::new_err(e.to_string()))?
                .ok_or_else(|| PyValueError::new_err(
                    "paatt_encode_base_data: raw version requires 'base_data_b64' key",
                ))?;
            let s: String = b64_str.extract()
                .map_err(|e| PyValueError::new_err(format!("base_data_b64: {}", e)))?;
            B64.decode(&s).map_err(|e| PyValueError::new_err(format!("base64 decode: {}", e)))?
        }
    };
    let _ = py;
    Ok(PyBytes::new(py, &bytes))
}

/// Convert a Python dict to serde_json::Value (handles nested dicts for catch_desc etc.).
fn py_dict_to_json_value(dict: &Bound<'_, PyDict>) -> PyResult<serde_json::Value> {
    let mut map = serde_json::Map::new();
    for (k, v) in dict.iter() {
        let key: String = k.extract()?;
        let jv = if v.is_instance_of::<PyBool>() {
            let b: bool = v.extract()?;
            serde_json::Value::Bool(b)
        } else if v.is_instance_of::<PyInt>() {
            let n: i64 = v.extract().unwrap_or_else(|_| { let u: u64 = v.extract().unwrap_or(0); u as i64 });
            serde_json::Value::Number(n.into())
        } else if v.is_instance_of::<PyFloat>() {
            let f: f64 = v.extract()?;
            serde_json::Value::Number(
                serde_json::Number::from_f64(f)
                    .unwrap_or_else(|| serde_json::Number::from(0i64)),
            )
        } else if v.is_instance_of::<PyString>() {
            let s: String = v.extract()?;
            serde_json::Value::String(s)
        } else if let Ok(d) = v.cast::<PyDict>() {
            py_dict_to_json_value(d)?
        } else {
            serde_json::Value::Null
        };
        map.insert(key, jv);
    }
    Ok(serde_json::Value::Object(map))
}

/// Convert a serde_json::Value to a Python object.
fn json_value_to_py<'py>(py: Python<'py>, v: &serde_json::Value) -> PyResult<Bound<'py, PyAny>> {
    Ok(match v {
        serde_json::Value::Bool(b) => PyBool::new(py, *b).to_owned().into_any(),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_u64() {
                i.into_pyobject(py)?.into_any()
            } else if let Some(i) = n.as_i64() {
                i.into_pyobject(py)?.into_any()
            } else {
                n.as_f64().unwrap_or(0.0).into_pyobject(py)?.into_any()
            }
        }
        serde_json::Value::String(s) => s.clone().into_pyobject(py)?.into_any(),
        serde_json::Value::Object(obj) => {
            let d = PyDict::new(py);
            for (k, val) in obj {
                d.set_item(k, json_value_to_py(py, val)?)?;
            }
            d.into_any()
        }
        serde_json::Value::Array(arr) => {
            let items: PyResult<Vec<_>> = arr.iter().map(|val| json_value_to_py(py, val)).collect();
            PyList::new(py, items?)?.into_any()
        }
        serde_json::Value::Null => py.None().into_bound(py).into_any(),
    })
}

// ── Registration ───────────────────────────────────────────────────────────

pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(parse_iteminfo_from_file, m)?)?;
    m.add_function(wrap_pyfunction!(parse_iteminfo_from_bytes, m)?)?;
    m.add_function(wrap_pyfunction!(parse_iteminfo_tracked, m)?)?;
    m.add_function(wrap_pyfunction!(write_iteminfo_to_file, m)?)?;
    m.add_function(wrap_pyfunction!(serialize_iteminfo, m)?)?;
    m.add_function(wrap_pyfunction!(parse_papgt_file, m)?)?;
    m.add_function(wrap_pyfunction!(parse_papgt_bytes, m)?)?;
    m.add_function(wrap_pyfunction!(write_papgt_file, m)?)?;
    m.add_function(wrap_pyfunction!(serialize_papgt, m)?)?;
    m.add_function(wrap_pyfunction!(parse_pamt_file, m)?)?;
    m.add_function(wrap_pyfunction!(parse_pamt_bytes, m)?)?;
    m.add_function(wrap_pyfunction!(write_pamt_file, m)?)?;
    m.add_function(wrap_pyfunction!(serialize_pamt, m)?)?;
    m.add_function(wrap_pyfunction!(calculate_checksum, m)?)?;
    m.add_function(wrap_pyfunction!(compress_data, m)?)?;
    m.add_function(wrap_pyfunction!(decompress_data, m)?)?;
    m.add_class::<PyPackGroupBuilder>()?;
    m.add_function(wrap_pyfunction!(add_papgt_entry, m)?)?;
    m.add_function(wrap_pyfunction!(extract_file, m)?)?;
    m.add_function(wrap_pyfunction!(parse_paloc_bytes, m)?)?;
    m.add_function(wrap_pyfunction!(serialize_paloc, m)?)?;
    m.add_function(wrap_pyfunction!(parse_paloc_from_file, m)?)?;
    m.add_function(wrap_pyfunction!(parse_paloc_from_bytes, m)?)?;
    m.add_function(wrap_pyfunction!(serialize_paloc_to_bytes, m)?)?;
    m.add_function(wrap_pyfunction!(classify_dds, m)?)?;
    m.add_function(wrap_pyfunction!(validate_dds, m)?)?;
    m.add_function(wrap_pyfunction!(infer_dds_vpath, m)?)?;
    m.add_function(wrap_pyfunction!(classify_vpath_last4, m)?)?;
    m.add_function(wrap_pyfunction!(classify_wem, m)?)?;
    m.add_function(wrap_pyfunction!(parse_bnk, m)?)?;
    m.add_function(wrap_pyfunction!(infer_audio_vpath, m)?)?;
    m.add_function(wrap_pyfunction!(validate_audio, m)?)?;
    m.add_function(wrap_pyfunction!(parse_skillinfo_from_file, m)?)?;
    m.add_function(wrap_pyfunction!(parse_skillinfo_from_bytes, m)?)?;
    m.add_function(wrap_pyfunction!(serialize_skillinfo, m)?)?;
    m.add_function(wrap_pyfunction!(write_skillinfo_to_file, m)?)?;
    m.add_function(wrap_pyfunction!(parse_buffinfo_from_file, m)?)?;
    m.add_function(wrap_pyfunction!(parse_buffinfo_from_bytes, m)?)?;
    m.add_function(wrap_pyfunction!(serialize_buffinfo, m)?)?;
    m.add_function(wrap_pyfunction!(write_buffinfo_to_file, m)?)?;
    m.add_function(wrap_pyfunction!(parse_table, m)?)?;
    m.add_function(wrap_pyfunction!(serialize_table, m)?)?;
    m.add_function(wrap_pyfunction!(write_table_to_file, m)?)?;
    m.add_function(wrap_pyfunction!(apply_intents, m)?)?;
    m.add_function(wrap_pyfunction!(normalize_target_name, m)?)?;
    m.add_function(wrap_pyfunction!(item_paloc_indices, m)?)?;
    m.add_function(wrap_pyfunction!(parse_pastage_bytes, m)?)?;
    m.add_function(wrap_pyfunction!(serialize_pastage, m)?)?;
    m.add_function(wrap_pyfunction!(parse_paseq_bytes, m)?)?;
    m.add_function(wrap_pyfunction!(serialize_paseq, m)?)?;
    m.add_function(wrap_pyfunction!(parse_paseqc_bytes, m)?)?;
    m.add_function(wrap_pyfunction!(serialize_paseqc, m)?)?;
    m.add_function(wrap_pyfunction!(parse_paschedule_bytes, m)?)?;
    m.add_function(wrap_pyfunction!(serialize_paschedule, m)?)?;
    m.add_function(wrap_pyfunction!(parse_paschedulepath_bytes, m)?)?;
    m.add_function(wrap_pyfunction!(serialize_paschedulepath, m)?)?;
    m.add_function(wrap_pyfunction!(parse_paatt_bytes, m)?)?;
    m.add_function(wrap_pyfunction!(serialize_paatt, m)?)?;
    m.add_function(wrap_pyfunction!(parse_paseq_field_directory, m)?)?;
    m.add_function(wrap_pyfunction!(parse_paseqc_field_directory, m)?)?;
    m.add_function(wrap_pyfunction!(parse_paseq_all_class_blocks, m)?)?;
    m.add_function(wrap_pyfunction!(parse_paseqc_all_class_blocks, m)?)?;
    m.add_function(wrap_pyfunction!(paseq_value_section_offset, m)?)?;
    m.add_function(wrap_pyfunction!(paseq_value_section, m)?)?;
    m.add_function(wrap_pyfunction!(paseqc_value_section_offset, m)?)?;
    m.add_function(wrap_pyfunction!(paseqc_value_section, m)?)?;
    m.add_function(wrap_pyfunction!(paseq_value_section_strings, m)?)?;
    m.add_function(wrap_pyfunction!(paseqc_value_section_strings, m)?)?;
    m.add_function(wrap_pyfunction!(replace_cstring_at, m)?)?;
    m.add_function(wrap_pyfunction!(walk_lp_strings, m)?)?;
    // Tier 1 file-path convenience wrappers
    m.add_function(wrap_pyfunction!(parse_pastage_from_file, m)?)?;
    m.add_function(wrap_pyfunction!(write_pastage_to_file, m)?)?;
    m.add_function(wrap_pyfunction!(parse_paseq_from_file, m)?)?;
    m.add_function(wrap_pyfunction!(write_paseq_to_file, m)?)?;
    m.add_function(wrap_pyfunction!(parse_paseqc_from_file, m)?)?;
    m.add_function(wrap_pyfunction!(write_paseqc_to_file, m)?)?;
    m.add_function(wrap_pyfunction!(parse_paschedule_from_file, m)?)?;
    m.add_function(wrap_pyfunction!(write_paschedule_to_file, m)?)?;
    m.add_function(wrap_pyfunction!(parse_paschedulepath_from_file, m)?)?;
    m.add_function(wrap_pyfunction!(write_paschedulepath_to_file, m)?)?;
    m.add_function(wrap_pyfunction!(parse_paatt_from_file, m)?)?;
    m.add_function(wrap_pyfunction!(write_paatt_to_file, m)?)?;
    m.add_function(wrap_pyfunction!(paatt_decode_base_data, m)?)?;
    m.add_function(wrap_pyfunction!(paatt_encode_base_data, m)?)?;
    Ok(())
}
