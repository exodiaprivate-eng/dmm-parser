//! pa::GameExpression, 2.04.00 wire format (flattened token list).
//!
//! 2.04 replaced the recursive expression tree (`game_expression.rs`) with a flat list of
//! tokens that point at each other by index. Mac readers:
//!
//!   GameExpression (`sub_102677CF0`): u8 presence; when set, the token list
//!   (`sub_10267B038`: u32 count + tokens) then a u32.
//!
//!   token (`sub_102675590`): u8 presence (0 = empty slot), u8 class, u8 sub-type, three u32
//!   (default -1; token indices), two GameValue operands, CString name, u8 flag, then the
//!   class's own fields (vtable slot 9):
//!     class 1 `GameExpression_MemberFunction` (`sub_102676A00`): u32, CString, u32, CString,
//!       CArray<u32>
//!     class 2 `GameExpression_Value` (`sub_102677764`): CString, CString
//!     other  `GameExpression_Op`: nothing (slot 9 is the no-op `sub_1000168CC`)
//!
//!   GameValue operand (`sub_102674CA4`): u32 kind; kind 1 = `GameValue_Primitive`
//!   (u8 value type: 0 u8, 1 u32, 2 f32, 3 CString); kind 2 = an object built by name
//!   (CString type name + CString value); any other kind has no payload.
//!
//! The Python surface goes through the JSON path, like `OptionalGameCondition`.

use crate::binary::*;
use crate::json_traits::{ToJsonValue, WriteJsonValue, get_field as json_get_field};
use serde_json::{Map, Value};
use std::io::{self, Write};

fn bad(msg: String) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, msg)
}

fn obj<'v>(v: &'v Value, what: &str) -> io::Result<&'v Map<String, Value>> {
    v.as_object().ok_or_else(|| bad(format!("{}: expected object", what)))
}

fn u64_of(v: &Value, what: &str) -> io::Result<u64> {
    v.as_u64().ok_or_else(|| bad(format!("{}: expected integer", what)))
}

/// `GameValue_Primitive` payload, selected by its value-type byte.
#[derive(Debug)]
pub enum GamePrimitive<'a> {
    U8(u8),
    U32(u32),
    F32(f32),
    Str(CString<'a>),
}

/// One GameValue operand of a token.
#[derive(Debug)]
pub enum GameValueOperand<'a> {
    /// kind 1.
    Primitive(GamePrimitive<'a>),
    /// kind 2: object created by type name.
    Object { type_name: CString<'a>, value: CString<'a> },
    /// any other kind: no payload.
    Empty(u32),
}

impl<'a> BinaryRead<'a> for GameValueOperand<'a> {
    fn read_from(data: &'a [u8], offset: &mut usize) -> io::Result<Self> {
        let kind = u32::read_from(data, offset)?;
        Ok(match kind {
            1 => GameValueOperand::Primitive(match u8::read_from(data, offset)? {
                0 => GamePrimitive::U8(u8::read_from(data, offset)?),
                1 => GamePrimitive::U32(u32::read_from(data, offset)?),
                2 => GamePrimitive::F32(f32::read_from(data, offset)?),
                3 => GamePrimitive::Str(CString::read_from(data, offset)?),
                t => return Err(bad(format!("GameValue_Primitive: unknown value type {}", t))),
            }),
            2 => GameValueOperand::Object {
                type_name: CString::read_from(data, offset)?,
                value: CString::read_from(data, offset)?,
            },
            k => GameValueOperand::Empty(k),
        })
    }
}

impl BinaryWrite for GameValueOperand<'_> {
    fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        match self {
            GameValueOperand::Primitive(p) => {
                1u32.write_to(w)?;
                match p {
                    GamePrimitive::U8(v) => { 0u8.write_to(w)?; v.write_to(w) }
                    GamePrimitive::U32(v) => { 1u8.write_to(w)?; v.write_to(w) }
                    GamePrimitive::F32(v) => { 2u8.write_to(w)?; v.write_to(w) }
                    GamePrimitive::Str(v) => { 3u8.write_to(w)?; v.write_to(w) }
                }
            }
            GameValueOperand::Object { type_name, value } => {
                2u32.write_to(w)?;
                type_name.write_to(w)?;
                value.write_to(w)
            }
            GameValueOperand::Empty(k) => k.write_to(w),
        }
    }
}

impl ToJsonValue for GameValueOperand<'_> {
    fn to_json_value(&self) -> Value {
        let mut m = Map::new();
        match self {
            GameValueOperand::Primitive(p) => {
                m.insert("kind".into(), 1u32.to_json_value());
                let (t, v) = match p {
                    GamePrimitive::U8(v) => (0u8, v.to_json_value()),
                    GamePrimitive::U32(v) => (1, v.to_json_value()),
                    GamePrimitive::F32(v) => (2, v.to_json_value()),
                    GamePrimitive::Str(v) => (3, v.to_json_value()),
                };
                m.insert("value_type".into(), t.to_json_value());
                m.insert("value".into(), v);
            }
            GameValueOperand::Object { type_name, value } => {
                m.insert("kind".into(), 2u32.to_json_value());
                m.insert("type_name".into(), type_name.to_json_value());
                m.insert("value".into(), value.to_json_value());
            }
            GameValueOperand::Empty(k) => {
                m.insert("kind".into(), k.to_json_value());
            }
        }
        Value::Object(m)
    }
}

impl WriteJsonValue for GameValueOperand<'_> {
    fn write_from_json(w: &mut Vec<u8>, v: &Value) -> io::Result<()> {
        let o = obj(v, "GameValue")?;
        let kind = u64_of(json_get_field(o, "kind")?, "GameValue.kind")? as u32;
        kind.write_to(w)?;
        match kind {
            1 => {
                let t = u64_of(json_get_field(o, "value_type")?, "GameValue.value_type")? as u8;
                t.write_to(w)?;
                let val = json_get_field(o, "value")?;
                match t {
                    0 => <u8 as WriteJsonValue>::write_from_json(w, val),
                    1 => <u32 as WriteJsonValue>::write_from_json(w, val),
                    2 => <f32 as WriteJsonValue>::write_from_json(w, val),
                    3 => <CString as WriteJsonValue>::write_from_json(w, val),
                    t => Err(bad(format!("GameValue_Primitive: unknown value type {}", t))),
                }
            }
            2 => {
                <CString as WriteJsonValue>::write_from_json(w, json_get_field(o, "type_name")?)?;
                <CString as WriteJsonValue>::write_from_json(w, json_get_field(o, "value")?)
            }
            _ => Ok(()),
        }
    }
}

/// Class-specific tail of a token (vtable slot 9).
#[derive(Debug)]
pub enum GameExpressionTokenBody<'a> {
    /// class 1, `GameExpression_MemberFunction`.
    MemberFunction {
        u32_52: u32,
        class_name: CString<'a>,
        u32_64: u32,
        function_name: CString<'a>,
        arg_list: CArray<u32>,
    },
    /// class 2, `GameExpression_Value`.
    Value { value_name: CString<'a>, type_name: CString<'a> },
    /// any other class, `GameExpression_Op`.
    Op,
}

#[derive(Debug)]
pub struct GameExpressionToken<'a> {
    pub class: u8,
    pub sub_type: u8,
    pub index_a: u32,
    pub index_b: u32,
    pub index_c: u32,
    pub lhs: GameValueOperand<'a>,
    pub rhs: GameValueOperand<'a>,
    pub name: CString<'a>,
    pub flag: u8,
    pub body: GameExpressionTokenBody<'a>,
}

impl<'a> BinaryRead<'a> for GameExpressionToken<'a> {
    fn read_from(data: &'a [u8], offset: &mut usize) -> io::Result<Self> {
        let class = u8::read_from(data, offset)?;
        let sub_type = u8::read_from(data, offset)?;
        let index_a = u32::read_from(data, offset)?;
        let index_b = u32::read_from(data, offset)?;
        let index_c = u32::read_from(data, offset)?;
        let lhs = GameValueOperand::read_from(data, offset)?;
        let rhs = GameValueOperand::read_from(data, offset)?;
        let name = CString::read_from(data, offset)?;
        let flag = u8::read_from(data, offset)?;
        let body = match class {
            1 => GameExpressionTokenBody::MemberFunction {
                u32_52: u32::read_from(data, offset)?,
                class_name: CString::read_from(data, offset)?,
                u32_64: u32::read_from(data, offset)?,
                function_name: CString::read_from(data, offset)?,
                arg_list: CArray::<u32>::read_from(data, offset)?,
            },
            2 => GameExpressionTokenBody::Value {
                value_name: CString::read_from(data, offset)?,
                type_name: CString::read_from(data, offset)?,
            },
            _ => GameExpressionTokenBody::Op,
        };
        Ok(Self { class, sub_type, index_a, index_b, index_c, lhs, rhs, name, flag, body })
    }
}

impl BinaryWrite for GameExpressionToken<'_> {
    fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        self.class.write_to(w)?;
        self.sub_type.write_to(w)?;
        self.index_a.write_to(w)?;
        self.index_b.write_to(w)?;
        self.index_c.write_to(w)?;
        self.lhs.write_to(w)?;
        self.rhs.write_to(w)?;
        self.name.write_to(w)?;
        self.flag.write_to(w)?;
        match &self.body {
            GameExpressionTokenBody::MemberFunction { u32_52, class_name, u32_64, function_name, arg_list } => {
                u32_52.write_to(w)?;
                class_name.write_to(w)?;
                u32_64.write_to(w)?;
                function_name.write_to(w)?;
                arg_list.write_to(w)
            }
            GameExpressionTokenBody::Value { value_name, type_name } => {
                value_name.write_to(w)?;
                type_name.write_to(w)
            }
            GameExpressionTokenBody::Op => Ok(()),
        }
    }
}

impl ToJsonValue for GameExpressionToken<'_> {
    fn to_json_value(&self) -> Value {
        let mut m = Map::new();
        m.insert("class".into(), self.class.to_json_value());
        m.insert("sub_type".into(), self.sub_type.to_json_value());
        m.insert("index_a".into(), self.index_a.to_json_value());
        m.insert("index_b".into(), self.index_b.to_json_value());
        m.insert("index_c".into(), self.index_c.to_json_value());
        m.insert("lhs".into(), self.lhs.to_json_value());
        m.insert("rhs".into(), self.rhs.to_json_value());
        m.insert("name".into(), self.name.to_json_value());
        m.insert("flag".into(), self.flag.to_json_value());
        match &self.body {
            GameExpressionTokenBody::MemberFunction { u32_52, class_name, u32_64, function_name, arg_list } => {
                m.insert("u32_52".into(), u32_52.to_json_value());
                m.insert("class_name".into(), class_name.to_json_value());
                m.insert("u32_64".into(), u32_64.to_json_value());
                m.insert("function_name".into(), function_name.to_json_value());
                m.insert("arg_list".into(), arg_list.to_json_value());
            }
            GameExpressionTokenBody::Value { value_name, type_name } => {
                m.insert("value_name".into(), value_name.to_json_value());
                m.insert("type_name".into(), type_name.to_json_value());
            }
            GameExpressionTokenBody::Op => {}
        }
        Value::Object(m)
    }
}

impl WriteJsonValue for GameExpressionToken<'_> {
    fn write_from_json(w: &mut Vec<u8>, v: &Value) -> io::Result<()> {
        let o = obj(v, "GameExpression token")?;
        let class = u64_of(json_get_field(o, "class")?, "token.class")? as u8;
        class.write_to(w)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(o, "sub_type")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(o, "index_a")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(o, "index_b")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(o, "index_c")?)?;
        GameValueOperand::write_from_json(w, json_get_field(o, "lhs")?)?;
        GameValueOperand::write_from_json(w, json_get_field(o, "rhs")?)?;
        <CString as WriteJsonValue>::write_from_json(w, json_get_field(o, "name")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(o, "flag")?)?;
        match class {
            1 => {
                <u32 as WriteJsonValue>::write_from_json(w, json_get_field(o, "u32_52")?)?;
                <CString as WriteJsonValue>::write_from_json(w, json_get_field(o, "class_name")?)?;
                <u32 as WriteJsonValue>::write_from_json(w, json_get_field(o, "u32_64")?)?;
                <CString as WriteJsonValue>::write_from_json(w, json_get_field(o, "function_name")?)?;
                <CArray<u32> as WriteJsonValue>::write_from_json(w, json_get_field(o, "arg_list")?)
            }
            2 => {
                <CString as WriteJsonValue>::write_from_json(w, json_get_field(o, "value_name")?)?;
                <CString as WriteJsonValue>::write_from_json(w, json_get_field(o, "type_name")?)
            }
            _ => Ok(()),
        }
    }
}

/// A 2.04 GameExpression: presence byte, then the token list and a trailing u32.
/// Token slots carry their own presence byte; an empty slot is JSON `null`.
#[derive(Debug)]
pub struct GameExpression204<'a> {
    pub body: Option<(Vec<Option<GameExpressionToken<'a>>>, u32)>,
}

impl<'a> BinaryRead<'a> for GameExpression204<'a> {
    fn read_from(data: &'a [u8], offset: &mut usize) -> io::Result<Self> {
        if u8::read_from(data, offset)? == 0 {
            return Ok(Self { body: None });
        }
        let count = u32::read_from(data, offset)? as usize;
        check_remaining(data, *offset, count)?;
        let mut tokens = Vec::with_capacity(count);
        for _ in 0..count {
            tokens.push(if u8::read_from(data, offset)? == 0 {
                None
            } else {
                Some(GameExpressionToken::read_from(data, offset)?)
            });
        }
        let tail = u32::read_from(data, offset)?;
        Ok(Self { body: Some((tokens, tail)) })
    }
}

impl BinaryWrite for GameExpression204<'_> {
    fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        let Some((tokens, tail)) = &self.body else { return 0u8.write_to(w) };
        1u8.write_to(w)?;
        (tokens.len() as u32).write_to(w)?;
        for t in tokens {
            match t {
                Some(t) => { 1u8.write_to(w)?; t.write_to(w)?; }
                None => 0u8.write_to(w)?,
            }
        }
        tail.write_to(w)
    }
}

impl<'a> BinaryReadTracked<'a> for GameExpression204<'a> {
    fn read_tracked(
        data: &'a [u8],
        offset: &mut usize,
        path: &mut String,
        ranges: &mut Vec<FieldRange>,
    ) -> io::Result<Self> {
        let start = *offset;
        let item = Self::read_from(data, offset)?;
        ranges.push(FieldRange { path: path.clone(), start, end: *offset, ty: "GameExpression204" });
        Ok(item)
    }
}

impl ToJsonValue for GameExpression204<'_> {
    fn to_json_value(&self) -> Value {
        let Some((tokens, tail)) = &self.body else { return Value::Null };
        let mut m = Map::new();
        m.insert("tokens".into(), Value::Array(tokens.iter().map(|t| match t {
            Some(t) => t.to_json_value(),
            None => Value::Null,
        }).collect()));
        m.insert("tail".into(), tail.to_json_value());
        Value::Object(m)
    }
}

impl WriteJsonValue for GameExpression204<'_> {
    fn write_from_json(w: &mut Vec<u8>, v: &Value) -> io::Result<()> {
        if v.is_null() {
            0u8.write_to(w)?;
            return Ok(());
        }
        let o = obj(v, "GameExpression")?;
        let tokens = json_get_field(o, "tokens")?.as_array()
            .ok_or_else(|| bad("GameExpression.tokens: expected array".into()))?;
        1u8.write_to(w)?;
        (tokens.len() as u32).write_to(w)?;
        for t in tokens {
            if t.is_null() {
                0u8.write_to(w)?;
            } else {
                1u8.write_to(w)?;
                GameExpressionToken::write_from_json(w, t)?;
            }
        }
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(o, "tail")?)
    }
}

impl crate::python_traits::ToPyValue for GameExpression204<'_> {
    fn to_py_value(&self, _py: pyo3::Python<'_>) -> pyo3::PyResult<pyo3::Py<pyo3::PyAny>> {
        Err(pyo3::exceptions::PyNotImplementedError::new_err("GameExpression204: use JSON path"))
    }
}

impl crate::python_traits::WritePyValue for GameExpression204<'_> {
    fn write_from_py(_w: &mut Vec<u8>, _obj: &pyo3::Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<()> {
        Err(pyo3::exceptions::PyNotImplementedError::new_err("GameExpression204: use JSON path"))
    }
}
