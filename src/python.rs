// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 §4.9 (No Competing Implementation) and §4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. §1202.

use pyo3::prelude::*;
use pyo3::types::{PyBool, PyBytes, PyDict, PyFloat, PyInt, PyList, PyNone};
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
    Ok(())
}
