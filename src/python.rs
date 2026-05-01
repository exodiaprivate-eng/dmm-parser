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

fn dispatch_parse(
    table_name: &str,
    pabgb: &[u8],
    pabgh: Option<&[u8]>,
) -> PyResult<Vec<serde_json::Value>> {
    use crate::binary::BinaryRead;
    use crate::tables::blob_runtime::parse_typed_blob_table_to_json_with_pabgh;

    macro_rules! p {
        ($ty:path) => {{
            let ph = pabgh.ok_or_else(|| PyValueError::new_err(
                format!("table '{}' requires a pabgh file", table_name)))?;
            parse_typed_blob_table_to_json_with_pabgh(pabgb, ph, |data, offset, size| {
                <$ty>::read_with_size(data, offset, size).map(|t| t.to_json_dict())
            }).map_err(|e| PyValueError::new_err(e.to_string()))?
        }};
    }

    macro_rules! s {
        ($ty:path) => {{
            let mut offset = 0usize;
            let mut out: Vec<serde_json::Value> = Vec::new();
            while offset < pabgb.len() {
                let item = <$ty>::read_from(pabgb, &mut offset)
                    .map_err(|e| PyValueError::new_err(
                        format!("offset 0x{:08x}: {}", offset, e)))?;
                out.push(serde_json::Value::Object(item.to_json_dict()));
            }
            out
        }};
    }

    Ok(match table_name {
        // ── pabgh-bounded tables ──────────────────────────────────────────
        "ai_dialog_string_info"          => p!(crate::tables::ai_dialog_string_info::AIDialogStringInfo),
        "bitmap_position_info"           => p!(crate::tables::bitmap_position_info::BitmapPositionInfo),
        "buff_info"                      => p!(crate::tables::buff_info::BuffInfo),
        "character_change_info"          => p!(crate::tables::character_change_info::CharacterChangeInfo),
        "character_info"                 => p!(crate::tables::character_info::CharacterInfo),
        "condition_info"                 => p!(crate::tables::condition_info::ConditionInfo),
        "drop_set_info"                  => p!(crate::tables::drop_set_info::DropSetInfo),
        "effect_info"                    => p!(crate::tables::effect_info::EffectInfo),
        "elemental_material_info"        => p!(crate::tables::elemental_material_info::ElementalMaterialInfo),
        "equip_info"                     => p!(crate::tables::equip_info::EquipInfo),
        "equip_slot_info"                => {
            let ph = pabgh.ok_or_else(|| PyValueError::new_err(
                "table 'equip_slot_info' requires a pabgh file"))?;
            crate::tables::equip_slot_info::parse_equip_slot_info_to_json_with_pabgh(pabgb, ph)
                .map_err(|e| PyValueError::new_err(e.to_string()))?
        },
        "faction_info"                   => p!(crate::tables::faction_info::FactionInfo),
        "faction_node_info"              => p!(crate::tables::faction_node_info::FactionNodeInfo),
        "faction_node_spawn_info"        => p!(crate::tables::faction_node_spawn_info::FactionNodeSpawnInfo),
        "faction_spawn_data_info"        => p!(crate::tables::faction_spawn_data_info::FactionSpawnDataInfo),
        "field_revive_info"              => p!(crate::tables::field_revive_info::FieldReviveInfo),
        "frame_event_attr_group_info"    => p!(crate::tables::frame_event_attr_group_info::FrameEventAttrGroupInfo),
        "game_event_handler_info"        => p!(crate::tables::game_event_handler_info::GameEventHandlerInfo),
        "game_global_effect_info"        => p!(crate::tables::game_global_effect_info::GameGlobalEffectInfo),
        "game_level_info"                => p!(crate::tables::game_level_info::GameLevelInfo),
        "game_play_trigger_info"         => p!(crate::tables::game_play_trigger_info::GamePlayTriggerInfo),
        "gimmick_group_info"             => p!(crate::tables::gimmick_group_info::GimmickGroupInfo),
        "gimmick_info"                   => p!(crate::tables::gimmick_info::GimmickInfo),
        "global_game_event_info"         => p!(crate::tables::global_game_event_info::GlobalGameEventInfo),
        "global_stage_sequencer_info"    => p!(crate::tables::global_stage_sequencer_info::GlobalStageSequencerInfo),
        "interaction_info"               => p!(crate::tables::interaction_info::InteractionInfo),
        "inventory_info"                 => p!(crate::tables::inventory_info::InventoryInfo),
        "item_use_info"                  => p!(crate::tables::item_use_info::ItemUseInfo),
        "knowledge_info"                 => p!(crate::tables::knowledge_info::KnowledgeInfo),
        "level_gimmick_scene_object_info" => p!(crate::tables::level_gimmick_scene_object_info::LevelGimmickSceneObjectInfo),
        "mini_game_data_info"            => p!(crate::tables::mini_game_data_info::MiniGameDataInfo),
        "mission_info"                   => p!(crate::tables::mission_info::MissionInfo),
        "multi_change_info"              => p!(crate::tables::multi_change_info::MultiChangeInfo),
        "npc_info"                       => p!(crate::tables::npc_info::NpcInfo),
        "platform_entitlement_info"      => p!(crate::tables::platform_entitlement_info::PlatformEntitlementInfo),
        "quest_info"                     => p!(crate::tables::quest_info::QuestInfo),
        "region_info"                    => p!(crate::tables::region_info::RegionInfo),
        "royal_supply_info"              => p!(crate::tables::royal_supply_info::RoyalSupplyInfo),
        "sequencer_spawn_info"           => p!(crate::tables::sequencer_spawn_info::SequencerSpawnInfo),
        "skill_info"                     => {
            let ph = pabgh.ok_or_else(|| PyValueError::new_err(
                "table 'skill_info' requires a pabgh file"))?;
            crate::tables::skill_info::parse_skill_to_json_with_pabgh(pabgb, ph)
                .map_err(|e| PyValueError::new_err(e.to_string()))?
        },
        "spawning_pool_auto_spawn_info"  => p!(crate::tables::spawning_pool_auto_spawn_info::SpawningPoolAutoSpawnInfo),
        "special_mode_info"              => p!(crate::tables::special_mode_info::SpecialModeInfo),
        "stage_info"                     => p!(crate::tables::stage_info::StageInfo),
        "store_info"                     => p!(crate::tables::store_info::StoreInfo),
        "sub_level_info"                 => p!(crate::tables::sub_level_info::SubLevelInfo),
        "terrain_region_auto_spawn_info" => p!(crate::tables::terrain_region_auto_spawn_info::TerrainRegionAutoSpawnInfo),

        // ── sequential tables ─────────────────────────────────────────────
        "action_point_info"              => s!(crate::tables::action_point_info::ActionPointInfo),
        "action_restriction_order_info"  => s!(crate::tables::action_restriction_order_info::ActionRestrictionOrderInfo),
        "aiaction_attribute_info"        => s!(crate::tables::aiaction_attribute_info::AIActionAttributeInfo),
        "aidialog_type_info"             => s!(crate::tables::aidialog_type_info::AIDialogTypeInfo),
        "aievent_table_info"             => s!(crate::tables::aievent_table_info::AIEventTableInfo),
        "aimemory_info"                  => s!(crate::tables::aimemory_info::AIMemoryInfo),
        "aimove_speed_info"              => s!(crate::tables::aimove_speed_info::AIMoveSpeedInfo),
        "ally_group_info"                => s!(crate::tables::ally_group_info::AllyGroupInfo),
        "auto_spawn_filter_info"         => s!(crate::tables::auto_spawn_filter_info::AutoSpawnFilterInfo),
        "board_info"                     => s!(crate::tables::board_info::BoardInfo),
        "breakable_object_info"          => s!(crate::tables::breakable_object_info::BreakableObjectInfo),
        "category_group_info"            => s!(crate::tables::category_group_info::CategoryGroupInfo),
        "category_info"                  => s!(crate::tables::category_info::CategoryInfo),
        "character_appearance_index_info" => s!(crate::tables::character_appearance_index_info::CharacterAppearanceIndexInfo),
        "character_group_info"           => s!(crate::tables::character_group_info::CharacterGroupInfo),
        "craft_tool_group_info"          => s!(crate::tables::craft_tool_group_info::CraftToolGroupInfo),
        "craft_tool_info"                => s!(crate::tables::craft_tool_info::CraftToolInfo),
        "detect_detail_info"             => s!(crate::tables::detect_detail_info::DetectDetailInfo),
        "detect_info"                    => s!(crate::tables::detect_info::DetectInfo),
        "detect_reaction_info"           => s!(crate::tables::detect_reaction_info::DetectReactionInfo),
        "dialog_voice_info"              => s!(crate::tables::dialog_voice_info::DialogVoiceInfo),
        "dye_color_group_info"           => s!(crate::tables::dye_color_group_info::DyeColorGroupInfo),
        "equip_type_info"                => s!(crate::tables::equip_type_info::EquipTypeInfo),
        "faction_group_info"             => s!(crate::tables::faction_group_info::FactionGroupInfo),
        "faction_relation_group_info"    => s!(crate::tables::faction_relation_group_info::FactionRelationGroupInfo),
        "faction_waypoint_info"          => s!(crate::tables::faction_waypoint_info::FactionWaypointInfo),
        "fail_message_info"              => s!(crate::tables::fail_message_info::FailMessageInfo),
        "field_info"                     => s!(crate::tables::field_info::FieldInfo),
        "field_level_name_table_info"    => s!(crate::tables::field_level_name_table_info::FieldLevelNameTableInfo),
        "formation_info"                 => s!(crate::tables::formation_info::FormationInfo),
        "game_advice_group_info"         => s!(crate::tables::game_advice_group_info::GameAdviceGroupInfo),
        "game_advice_info"               => s!(crate::tables::game_advice_info::GameAdviceInfo),
        "game_play_variable_info"        => s!(crate::tables::game_play_variable_info::GamePlayVariableInfo),
        "gimmick_event_table_info"       => s!(crate::tables::gimmick_event_table_info::GimmickEventTableInfo),
        "gimmick_gate_connection_info"   => s!(crate::tables::gimmick_gate_connection_info::GimmickGateConnectionInfo),
        "gimmick_gate_info"              => s!(crate::tables::gimmick_gate_info::GimmickGateInfo),
        "global_game_event_group_info"   => s!(crate::tables::global_game_event_group_info::GlobalGameEventGroupInfo),
        "house_info"                     => s!(crate::tables::house_info::HouseInfo),
        "item_group_info"                => s!(crate::tables::item_group_info::ItemGroupInfo),
        "job_info"                       => s!(crate::tables::job_info::JobInfo),
        "key_map_setting_list_info"      => s!(crate::tables::key_map_setting_list_info::KeyMapSettingListInfo),
        "knowledge_group_info"           => s!(crate::tables::knowledge_group_info::KnowledgeGroupInfo),
        "level_action_point_info"        => s!(crate::tables::level_action_point_info::LevelActionPointInfo),
        "local_string_info"              => s!(crate::tables::local_string_info::LocalStringInfo),
        "material_blood_decal_info"      => s!(crate::tables::material_blood_decal_info::MaterialBloodDecalInfo),
        "material_match_info"            => s!(crate::tables::material_match_info::MaterialMatchInfo),
        "material_relation_info"         => s!(crate::tables::material_relation_info::MaterialRelationInfo),
        "mercenary_group_info"           => s!(crate::tables::mercenary_group_info::MercenaryGroupInfo),
        "mercenary_info"                 => s!(crate::tables::mercenary_info::MercenaryInfo),
        "part_prefab_dye_slot_info"      => s!(crate::tables::part_prefab_dye_slot_info::PartPrefabDyeSlotInfo),
        "part_prefab_dye_texture_pallete_info" => s!(crate::tables::part_prefab_dye_texture_pallete_info::PartPrefabDyeTexturePalleteInfo),
        "pattern_description_info"       => s!(crate::tables::pattern_description_info::PatternDescriptionInfo),
        "platform_achievement_info"      => s!(crate::tables::platform_achievement_info::PlatformAchievementInfo),
        "quest_gauge_info"               => s!(crate::tables::quest_gauge_info::QuestGaugeInfo),
        "quest_group_info"               => s!(crate::tables::quest_group_info::QuestGroupInfo),
        "quick_time_event_info"          => s!(crate::tables::quick_time_event_info::QuickTimeEventInfo),
        "relation_info"                  => s!(crate::tables::relation_info::RelationInfo),
        "reserve_slot_info"              => s!(crate::tables::reserve_slot_info::ReserveSlotInfo),
        "skill_group_info"               => s!(crate::tables::skill_group_info::SkillGroupInfo),
        "skill_tree_group_info"          => s!(crate::tables::skill_tree_group_info::SkillTreeGroupInfo),
        "skill_tree_info"                => s!(crate::tables::skill_tree_info::SkillTreeInfo),
        "socket_group_info"              => s!(crate::tables::socket_group_info::SocketGroupInfo),
        "socket_info"                    => s!(crate::tables::socket_info::SocketInfo),
        "status_group_info"              => s!(crate::tables::status_group_info::StatusGroupInfo),
        "status_info"                    => s!(crate::tables::status_info::StatusInfo),
        "string_info"                    => s!(crate::tables::string_info::StringInfo),
        "terrain_region_navi_info"       => s!(crate::tables::terrain_region_navi_info::TerrainRegionNaviInfo),
        "tribe_info"                     => s!(crate::tables::tribe_info::TribeInfo),
        "trigger_region_info"            => s!(crate::tables::trigger_region_info::TriggerRegionInfo),
        "ui_social_action_info"          => s!(crate::tables::ui_social_action_info::UISocialActionInfo),
        "uifilter_group_info"            => s!(crate::tables::uifilter_group_info::UIFilterGroupInfo),
        "uimap_texture_info"             => s!(crate::tables::uimap_texture_info::UIMapTextureInfo),
        "valid_schedule_action_info"     => s!(crate::tables::valid_schedule_action_info::ValidScheduleActionInfo),
        "vehicle_info"                   => s!(crate::tables::vehicle_info::VehicleInfo),
        "vibrate_pattern_info"           => s!(crate::tables::vibrate_pattern_info::VibratePatternInfo),
        "wanted_info"                    => s!(crate::tables::wanted_info::WantedInfo),

        _ => return Err(PyValueError::new_err(format!("unknown table: '{}'", table_name))),
    })
}

fn dispatch_serialize_bytes(
    table_name: &str,
    json_items: &[serde_json::Value],
) -> PyResult<Vec<u8>> {
    use crate::tables::blob_runtime::serialize_typed_blob_table_from_json;

    macro_rules! d {
        ($ty:path) => {
            serialize_typed_blob_table_from_json(json_items, |w, map| {
                <$ty>::write_from_json_dict(w, map)
            }).map_err(|e| PyValueError::new_err(e.to_string()))?
        };
    }

    Ok(match table_name {
        // ── pabgh-bounded tables ──────────────────────────────────────────
        "ai_dialog_string_info"          => d!(crate::tables::ai_dialog_string_info::AIDialogStringInfo),
        "bitmap_position_info"           => d!(crate::tables::bitmap_position_info::BitmapPositionInfo),
        "buff_info"                      => d!(crate::tables::buff_info::BuffInfo),
        "character_change_info"          => d!(crate::tables::character_change_info::CharacterChangeInfo),
        "character_info"                 => d!(crate::tables::character_info::CharacterInfo),
        "condition_info"                 => d!(crate::tables::condition_info::ConditionInfo),
        "drop_set_info"                  => d!(crate::tables::drop_set_info::DropSetInfo),
        "effect_info"                    => d!(crate::tables::effect_info::EffectInfo),
        "elemental_material_info"        => d!(crate::tables::elemental_material_info::ElementalMaterialInfo),
        "equip_info"                     => d!(crate::tables::equip_info::EquipInfo),
        "equip_slot_info"                => {
            crate::tables::equip_slot_info::serialize_equip_slot_info_from_json(json_items)
                .map_err(|e| PyValueError::new_err(e.to_string()))?
        },
        "faction_info"                   => d!(crate::tables::faction_info::FactionInfo),
        "faction_node_info"              => d!(crate::tables::faction_node_info::FactionNodeInfo),
        "faction_node_spawn_info"        => d!(crate::tables::faction_node_spawn_info::FactionNodeSpawnInfo),
        "faction_spawn_data_info"        => d!(crate::tables::faction_spawn_data_info::FactionSpawnDataInfo),
        "field_revive_info"              => d!(crate::tables::field_revive_info::FieldReviveInfo),
        "frame_event_attr_group_info"    => d!(crate::tables::frame_event_attr_group_info::FrameEventAttrGroupInfo),
        "game_event_handler_info"        => d!(crate::tables::game_event_handler_info::GameEventHandlerInfo),
        "game_global_effect_info"        => d!(crate::tables::game_global_effect_info::GameGlobalEffectInfo),
        "game_level_info"                => d!(crate::tables::game_level_info::GameLevelInfo),
        "game_play_trigger_info"         => d!(crate::tables::game_play_trigger_info::GamePlayTriggerInfo),
        "gimmick_group_info"             => d!(crate::tables::gimmick_group_info::GimmickGroupInfo),
        "gimmick_info"                   => d!(crate::tables::gimmick_info::GimmickInfo),
        "global_game_event_info"         => d!(crate::tables::global_game_event_info::GlobalGameEventInfo),
        "global_stage_sequencer_info"    => d!(crate::tables::global_stage_sequencer_info::GlobalStageSequencerInfo),
        "interaction_info"               => d!(crate::tables::interaction_info::InteractionInfo),
        "inventory_info"                 => d!(crate::tables::inventory_info::InventoryInfo),
        "item_use_info"                  => d!(crate::tables::item_use_info::ItemUseInfo),
        "knowledge_info"                 => d!(crate::tables::knowledge_info::KnowledgeInfo),
        "level_gimmick_scene_object_info" => d!(crate::tables::level_gimmick_scene_object_info::LevelGimmickSceneObjectInfo),
        "mini_game_data_info"            => d!(crate::tables::mini_game_data_info::MiniGameDataInfo),
        "mission_info"                   => d!(crate::tables::mission_info::MissionInfo),
        "multi_change_info"              => d!(crate::tables::multi_change_info::MultiChangeInfo),
        "npc_info"                       => d!(crate::tables::npc_info::NpcInfo),
        "platform_entitlement_info"      => d!(crate::tables::platform_entitlement_info::PlatformEntitlementInfo),
        "quest_info"                     => d!(crate::tables::quest_info::QuestInfo),
        "region_info"                    => d!(crate::tables::region_info::RegionInfo),
        "royal_supply_info"              => d!(crate::tables::royal_supply_info::RoyalSupplyInfo),
        "sequencer_spawn_info"           => d!(crate::tables::sequencer_spawn_info::SequencerSpawnInfo),
        "skill_info"                     => {
            crate::tables::skill_info::serialize_skill_from_json(json_items)
                .map_err(|e| PyValueError::new_err(e.to_string()))?
        },
        "spawning_pool_auto_spawn_info"  => d!(crate::tables::spawning_pool_auto_spawn_info::SpawningPoolAutoSpawnInfo),
        "special_mode_info"              => d!(crate::tables::special_mode_info::SpecialModeInfo),
        "stage_info"                     => d!(crate::tables::stage_info::StageInfo),
        "store_info"                     => d!(crate::tables::store_info::StoreInfo),
        "sub_level_info"                 => d!(crate::tables::sub_level_info::SubLevelInfo),
        "terrain_region_auto_spawn_info" => d!(crate::tables::terrain_region_auto_spawn_info::TerrainRegionAutoSpawnInfo),

        // ── sequential tables ─────────────────────────────────────────────
        "action_point_info"              => d!(crate::tables::action_point_info::ActionPointInfo),
        "action_restriction_order_info"  => d!(crate::tables::action_restriction_order_info::ActionRestrictionOrderInfo),
        "aiaction_attribute_info"        => d!(crate::tables::aiaction_attribute_info::AIActionAttributeInfo),
        "aidialog_type_info"             => d!(crate::tables::aidialog_type_info::AIDialogTypeInfo),
        "aievent_table_info"             => d!(crate::tables::aievent_table_info::AIEventTableInfo),
        "aimemory_info"                  => d!(crate::tables::aimemory_info::AIMemoryInfo),
        "aimove_speed_info"              => d!(crate::tables::aimove_speed_info::AIMoveSpeedInfo),
        "ally_group_info"                => d!(crate::tables::ally_group_info::AllyGroupInfo),
        "auto_spawn_filter_info"         => d!(crate::tables::auto_spawn_filter_info::AutoSpawnFilterInfo),
        "board_info"                     => d!(crate::tables::board_info::BoardInfo),
        "breakable_object_info"          => d!(crate::tables::breakable_object_info::BreakableObjectInfo),
        "category_group_info"            => d!(crate::tables::category_group_info::CategoryGroupInfo),
        "category_info"                  => d!(crate::tables::category_info::CategoryInfo),
        "character_appearance_index_info" => d!(crate::tables::character_appearance_index_info::CharacterAppearanceIndexInfo),
        "character_group_info"           => d!(crate::tables::character_group_info::CharacterGroupInfo),
        "craft_tool_group_info"          => d!(crate::tables::craft_tool_group_info::CraftToolGroupInfo),
        "craft_tool_info"                => d!(crate::tables::craft_tool_info::CraftToolInfo),
        "detect_detail_info"             => d!(crate::tables::detect_detail_info::DetectDetailInfo),
        "detect_info"                    => d!(crate::tables::detect_info::DetectInfo),
        "detect_reaction_info"           => d!(crate::tables::detect_reaction_info::DetectReactionInfo),
        "dialog_voice_info"              => d!(crate::tables::dialog_voice_info::DialogVoiceInfo),
        "dye_color_group_info"           => d!(crate::tables::dye_color_group_info::DyeColorGroupInfo),
        "equip_type_info"                => d!(crate::tables::equip_type_info::EquipTypeInfo),
        "faction_group_info"             => d!(crate::tables::faction_group_info::FactionGroupInfo),
        "faction_relation_group_info"    => d!(crate::tables::faction_relation_group_info::FactionRelationGroupInfo),
        "faction_waypoint_info"          => d!(crate::tables::faction_waypoint_info::FactionWaypointInfo),
        "fail_message_info"              => d!(crate::tables::fail_message_info::FailMessageInfo),
        "field_info"                     => d!(crate::tables::field_info::FieldInfo),
        "field_level_name_table_info"    => d!(crate::tables::field_level_name_table_info::FieldLevelNameTableInfo),
        "formation_info"                 => d!(crate::tables::formation_info::FormationInfo),
        "game_advice_group_info"         => d!(crate::tables::game_advice_group_info::GameAdviceGroupInfo),
        "game_advice_info"               => d!(crate::tables::game_advice_info::GameAdviceInfo),
        "game_play_variable_info"        => d!(crate::tables::game_play_variable_info::GamePlayVariableInfo),
        "gimmick_event_table_info"       => d!(crate::tables::gimmick_event_table_info::GimmickEventTableInfo),
        "gimmick_gate_connection_info"   => d!(crate::tables::gimmick_gate_connection_info::GimmickGateConnectionInfo),
        "gimmick_gate_info"              => d!(crate::tables::gimmick_gate_info::GimmickGateInfo),
        "global_game_event_group_info"   => d!(crate::tables::global_game_event_group_info::GlobalGameEventGroupInfo),
        "house_info"                     => d!(crate::tables::house_info::HouseInfo),
        "item_group_info"                => d!(crate::tables::item_group_info::ItemGroupInfo),
        "job_info"                       => d!(crate::tables::job_info::JobInfo),
        "key_map_setting_list_info"      => d!(crate::tables::key_map_setting_list_info::KeyMapSettingListInfo),
        "knowledge_group_info"           => d!(crate::tables::knowledge_group_info::KnowledgeGroupInfo),
        "level_action_point_info"        => d!(crate::tables::level_action_point_info::LevelActionPointInfo),
        "local_string_info"              => d!(crate::tables::local_string_info::LocalStringInfo),
        "material_blood_decal_info"      => d!(crate::tables::material_blood_decal_info::MaterialBloodDecalInfo),
        "material_match_info"            => d!(crate::tables::material_match_info::MaterialMatchInfo),
        "material_relation_info"         => d!(crate::tables::material_relation_info::MaterialRelationInfo),
        "mercenary_group_info"           => d!(crate::tables::mercenary_group_info::MercenaryGroupInfo),
        "mercenary_info"                 => d!(crate::tables::mercenary_info::MercenaryInfo),
        "part_prefab_dye_slot_info"      => d!(crate::tables::part_prefab_dye_slot_info::PartPrefabDyeSlotInfo),
        "part_prefab_dye_texture_pallete_info" => d!(crate::tables::part_prefab_dye_texture_pallete_info::PartPrefabDyeTexturePalleteInfo),
        "pattern_description_info"       => d!(crate::tables::pattern_description_info::PatternDescriptionInfo),
        "platform_achievement_info"      => d!(crate::tables::platform_achievement_info::PlatformAchievementInfo),
        "quest_gauge_info"               => d!(crate::tables::quest_gauge_info::QuestGaugeInfo),
        "quest_group_info"               => d!(crate::tables::quest_group_info::QuestGroupInfo),
        "quick_time_event_info"          => d!(crate::tables::quick_time_event_info::QuickTimeEventInfo),
        "relation_info"                  => d!(crate::tables::relation_info::RelationInfo),
        "reserve_slot_info"              => d!(crate::tables::reserve_slot_info::ReserveSlotInfo),
        "skill_group_info"               => d!(crate::tables::skill_group_info::SkillGroupInfo),
        "skill_tree_group_info"          => d!(crate::tables::skill_tree_group_info::SkillTreeGroupInfo),
        "skill_tree_info"                => d!(crate::tables::skill_tree_info::SkillTreeInfo),
        "socket_group_info"              => d!(crate::tables::socket_group_info::SocketGroupInfo),
        "socket_info"                    => d!(crate::tables::socket_info::SocketInfo),
        "status_group_info"              => d!(crate::tables::status_group_info::StatusGroupInfo),
        "status_info"                    => d!(crate::tables::status_info::StatusInfo),
        "string_info"                    => d!(crate::tables::string_info::StringInfo),
        "terrain_region_navi_info"       => d!(crate::tables::terrain_region_navi_info::TerrainRegionNaviInfo),
        "tribe_info"                     => d!(crate::tables::tribe_info::TribeInfo),
        "trigger_region_info"            => d!(crate::tables::trigger_region_info::TriggerRegionInfo),
        "ui_social_action_info"          => d!(crate::tables::ui_social_action_info::UISocialActionInfo),
        "uifilter_group_info"            => d!(crate::tables::uifilter_group_info::UIFilterGroupInfo),
        "uimap_texture_info"             => d!(crate::tables::uimap_texture_info::UIMapTextureInfo),
        "valid_schedule_action_info"     => d!(crate::tables::valid_schedule_action_info::ValidScheduleActionInfo),
        "vehicle_info"                   => d!(crate::tables::vehicle_info::VehicleInfo),
        "vibrate_pattern_info"           => d!(crate::tables::vibrate_pattern_info::VibratePatternInfo),
        "wanted_info"                    => d!(crate::tables::wanted_info::WantedInfo),

        _ => return Err(PyValueError::new_err(format!("unknown table: '{}'", table_name))),
    })
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
