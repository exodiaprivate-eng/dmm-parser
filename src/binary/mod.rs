// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 §4.9 (No Competing Implementation) and §4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. §1202.

mod primitives;
mod types;
mod arrays;
pub(crate) mod trie;
pub mod pabgh;
pub mod papgt;
pub mod pamt;
pub mod paz;
pub mod paloc;
pub mod variant;
pub mod variants;
pub mod optional_game_condition;
pub mod paatt;
pub mod paatt_basedata;
pub mod lp_token_stream;
pub mod pastage;
pub mod paseq;
pub mod paseqc;
pub mod paschedule;
pub mod paschedulepath;
pub mod dds;
pub mod wem;
pub mod bnk;
pub mod pami;
pub mod par_resource;
pub mod motionblending;
pub mod pamlod;
pub mod hkx;
pub mod paasmt;
pub mod paccd;
pub mod binarystring;
pub mod xml_resource;
pub mod xml_patch;
pub mod impostor;
pub mod count_record_table;

pub use types::*;

use std::io::{self, Write};

// ── Traits ──────────────────────────────────────────────────────────────────

pub trait BinaryRead<'a>: Sized {
    fn read_from(data: &'a [u8], offset: &mut usize) -> io::Result<Self>;
}

pub trait BinaryWrite {
    fn write_to(&self, writer: &mut dyn Write) -> io::Result<()>;
}

// ── Range tracking (used to map file bytes → field paths) ───────────────────
//
// Parallel to `BinaryRead`. `read_tracked` walks the same bytes in the same
// order as `read_from`, but also records a `FieldRange` for every leaf
// consumed — so callers can answer "what field does byte N of entry X
// belong to?" with a binary-search lookup.
//
// `path` is a mutable buffer reused across recursion to avoid per-call
// allocation: children push a segment before recursing, then truncate back
// to the parent's length.

#[derive(Debug, Clone)]
pub struct FieldRange {
    pub path: String,
    pub start: usize,
    pub end: usize,
    pub ty: &'static str,
}

pub trait BinaryReadTracked<'a>: Sized {
    fn read_tracked(
        data: &'a [u8],
        offset: &mut usize,
        path: &mut String,
        ranges: &mut Vec<FieldRange>,
    ) -> io::Result<Self>;
}

/// Push a child segment onto `path`, returning the previous length so
/// the caller can restore it. Uses `.` separator except at the root.
#[inline]
pub(crate) fn push_path(path: &mut String, seg: &str) -> usize {
    let saved = path.len();
    if !path.is_empty() {
        path.push('.');
    }
    path.push_str(seg);
    saved
}

/// Push an array index `[i]` onto `path`.
#[inline]
pub(crate) fn push_index(path: &mut String, i: usize) -> usize {
    let saved = path.len();
    use std::fmt::Write as _;
    write!(path, "[{}]", i).expect("fmt to String");
    saved
}

#[inline]
pub(crate) fn pop_path(path: &mut String, saved: usize) {
    path.truncate(saved);
}

// Public re-exports of path helpers so per-table hand-written
// `read_tracked_with_size` impls can use them without depending on the
// trait machinery. Hand-rolled Tier 1.5 tables (character_change_info,
// inventory_info, buff_info, condition_info, drop_set_info, effect_info,
// gimmick_info, interaction_info, store_info, faction_node_spawn_info,
// quest_info, item_use_info, ai_dialog_string_info,
// frame_event_attr_group_info, stage_info) record a top-level FieldRange
// per field rather than recursing through nested types. That's enough
// granularity for v2-byte-patch resolution: any patch whose offset falls
// inside a field's bytes can be resolved to that field name, and the
// converter promotes the patched bytes to a whole-field intent.
#[doc(hidden)]
pub fn _push_path_pub(path: &mut String, seg: &str) -> usize {
    push_path(path, seg)
}
#[doc(hidden)]
pub fn _pop_path_pub(path: &mut String, saved: usize) {
    pop_path(path, saved)
}

/// Helper for hand-rolled `read_tracked_with_size` impls. Reads one
/// field via `BinaryRead`, registers its byte span as one
/// `FieldRange`, and returns the value. Use for primitives and for
/// types where field-level granularity is sufficient (containers like
/// `Vec<CString>` get tracked as one opaque field; that's still
/// enough to route a v2 byte-patch to the correct field name).
pub fn track_read_field<'a, T>(
    data: &'a [u8],
    offset: &mut usize,
    path: &mut String,
    ranges: &mut Vec<FieldRange>,
    field_name: &str,
    ty_name: &'static str,
) -> io::Result<T>
where
    T: BinaryRead<'a>,
{
    let saved = push_path(path, field_name);
    let start = *offset;
    let v = T::read_from(data, offset)?;
    let end = *offset;
    ranges.push(FieldRange {
        path: path.clone(),
        start,
        end,
        ty: ty_name,
    });
    pop_path(path, saved);
    Ok(v)
}

/// Same as `track_read_field` but for fields read via a custom closure
/// rather than `BinaryRead`. Use when the field's bytes are consumed
/// by a custom helper (e.g. Vec<CString> via for-loop, polymorphic
/// CArray with per-variant body).
pub fn track_read_with<'a, T, F>(
    offset: &mut usize,
    path: &mut String,
    ranges: &mut Vec<FieldRange>,
    field_name: &str,
    ty_name: &'static str,
    read_fn: F,
) -> io::Result<T>
where
    F: FnOnce(&mut usize) -> io::Result<T>,
{
    let saved = push_path(path, field_name);
    let start = *offset;
    let v = read_fn(offset)?;
    let end = *offset;
    ranges.push(FieldRange {
        path: path.clone(),
        start,
        end,
        ty: ty_name,
    });
    pop_path(path, saved);
    Ok(v)
}

// ── Helpers ─────────────────────────────────────────────────────────────────

pub(crate) fn check_remaining(data: &[u8], offset: usize, need: usize) -> io::Result<()> {
    if offset + need > data.len() {
        Err(io::Error::new(io::ErrorKind::UnexpectedEof, "not enough data"))
    } else {
        Ok(())
    }
}

// ── Macro for simple structs (binary only, no Python conversion) ────────────

#[macro_export]
macro_rules! binary_struct {
    (
        $(#[$meta:meta])*
        pub struct $name:ident $(<$lt:lifetime>)? {
            $(pub $field:ident : $ty:ty),* $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug)]
        pub struct $name $(<$lt>)? {
            $(pub $field: $ty),*
        }

        impl<'a> $crate::binary::BinaryRead<'a> for $name $(<$lt>)? {
            fn read_from(data: &'a [u8], offset: &mut usize) -> std::io::Result<Self> {
                Ok($name {
                    $($field: $crate::binary::BinaryRead::read_from(data, offset)?),*
                })
            }
        }

        impl $(< $lt >)? $crate::binary::BinaryWrite for $name $(< $lt >)? {
            fn write_to(&self, w: &mut dyn std::io::Write) -> std::io::Result<()> {
                $($crate::binary::BinaryWrite::write_to(&self.$field, w)?;)*
                Ok(())
            }
        }
    };
}

// ── Macro for structs with binary + Python conversion ───────────────────────

#[macro_export]
macro_rules! py_binary_struct {
    (
        $(#[$meta:meta])*
        pub struct $name:ident $(<$lt:lifetime>)? {
            $(pub $field:ident : $ty:ty),* $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug)]
        pub struct $name $(<$lt>)? {
            $(pub $field: $ty),*
        }

        impl<'a> $crate::binary::BinaryRead<'a> for $name $(<$lt>)? {
            fn read_from(data: &'a [u8], offset: &mut usize) -> std::io::Result<Self> {
                Ok($name {
                    $($field: $crate::binary::BinaryRead::read_from(data, offset)?),*
                })
            }
        }

        impl<'a> $crate::binary::BinaryReadTracked<'a> for $name $(<$lt>)? {
            fn read_tracked(
                data: &'a [u8],
                offset: &mut usize,
                path: &mut String,
                ranges: &mut Vec<$crate::binary::FieldRange>,
            ) -> std::io::Result<Self> {
                Ok($name {
                    $(
                        $field: {
                            let __saved = $crate::binary::push_path(path, stringify!($field));
                            let __v = <$ty as $crate::binary::BinaryReadTracked>::read_tracked(
                                data, offset, path, ranges,
                            )?;
                            $crate::binary::pop_path(path, __saved);
                            __v
                        }
                    ),*
                })
            }
        }

        impl $(< $lt >)? $crate::binary::BinaryWrite for $name $(< $lt >)? {
            fn write_to(&self, w: &mut dyn std::io::Write) -> std::io::Result<()> {
                $($crate::binary::BinaryWrite::write_to(&self.$field, w)?;)*
                Ok(())
            }
        }

        impl $(< $lt >)? $name $(< $lt >)? {
            pub fn to_py_dict<'py>(&self, py: pyo3::Python<'py>)
                -> pyo3::PyResult<pyo3::Bound<'py, pyo3::types::PyDict>>
            {
                use $crate::python_traits::ToPyValue;
                use pyo3::types::PyDictMethods;
                let d = pyo3::types::PyDict::new(py);
                $(d.set_item(stringify!($field), self.$field.to_py_value(py)?)?;)*
                Ok(d)
            }

            pub fn write_from_py_dict(
                w: &mut Vec<u8>,
                d: &pyo3::Bound<'_, pyo3::types::PyDict>,
            ) -> pyo3::PyResult<()> {
                use $crate::python_traits::{WritePyValue, get_field};
                $(<$ty as WritePyValue>::write_from_py(w, &get_field(d, stringify!($field))?)?;)*
                Ok(())
            }

            // JSON-value mirror of the dict bridge above. Mod managers
            // running pure-Rust (no embedded Python) read the live binary
            // into a `serde_json::Value` tree, mutate it by field path,
            // then serialize back through `write_from_json_dict`. Field
            // names match Python's exactly.
            pub fn to_json_dict(&self) -> ::serde_json::Map<String, ::serde_json::Value> {
                #[allow(unused_imports)] use $crate::json_traits::ToJsonValue;
                let mut d = ::serde_json::Map::new();
                $(d.insert(stringify!($field).to_string(), self.$field.to_json_value());)*
                d
            }

            pub fn write_from_json_dict(
                w: &mut Vec<u8>,
                d: &::serde_json::Map<String, ::serde_json::Value>,
            ) -> ::std::io::Result<()> {
                // get_field_or_null returns Value::Null for absent keys rather
                // than erroring. This keeps old V3 mods working when a game
                // patch adds new struct fields (e.g. DockingChildData::unk_docking_108):
                // the missing field serializes as 0 via the primitive null-default
                // path instead of aborting the whole iteminfo overlay.
                use $crate::json_traits::{WriteJsonValue, get_field_or_null};
                $({
                    let _dmm_fval = get_field_or_null(d, stringify!($field));
                    <$ty as WriteJsonValue>::write_from_json(w, &_dmm_fval)
                        .map_err(|e| ::std::io::Error::new(e.kind(),
                            format!("{}.{}: {}", stringify!($name), stringify!($field), e)))?;
                })*
                Ok(())
            }
        }

        impl $(< $lt >)? $crate::python_traits::ToPyValue for $name $(< $lt >)? {
            fn to_py_value(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<pyo3::Py<pyo3::PyAny>> {
                Ok(self.to_py_dict(py)?.into_any().unbind())
            }
        }

        impl $(< $lt >)? $crate::python_traits::WritePyValue for $name $(< $lt >)? {
            fn write_from_py(w: &mut Vec<u8>, obj: &pyo3::Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<()> {
                Self::write_from_py_dict(w, obj.cast::<pyo3::types::PyDict>()?)
            }
        }

        impl $(< $lt >)? $crate::json_traits::ToJsonValue for $name $(< $lt >)? {
            fn to_json_value(&self) -> ::serde_json::Value {
                ::serde_json::Value::Object(self.to_json_dict())
            }
        }

        impl $(< $lt >)? $crate::json_traits::WriteJsonValue for $name $(< $lt >)? {
            fn write_from_json(w: &mut Vec<u8>, v: &::serde_json::Value) -> ::std::io::Result<()> {
                let obj = v.as_object().ok_or_else(|| ::std::io::Error::new(
                    ::std::io::ErrorKind::InvalidData,
                    format!("expected object for {}, got {:?}",
                        stringify!($name),
                        match v {
                            ::serde_json::Value::Null => "null",
                            ::serde_json::Value::Bool(_) => "bool",
                            ::serde_json::Value::Number(_) => "number",
                            ::serde_json::Value::String(_) => "string",
                            ::serde_json::Value::Array(_) => "array",
                            ::serde_json::Value::Object(_) => "object",
                        }),
                ))?;
                Self::write_from_json_dict(w, obj)
            }
        }
    };
}
