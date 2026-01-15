use std::{
    collections::{BTreeMap, HashMap},
    fmt::Display,
    path::PathBuf,
    str::FromStr,
    sync::Arc,
};

mod validator;

pub use validator::validate;

use crate::intermediate::{self, IntEnumVariant, StringEnumVariant};

#[derive(Debug)]
pub struct RawDocument {
    pub filepath: Option<PathBuf>,

    pub json_definitions: Vec<JsonDefinition>,

    pub http_definitions: Vec<HTTPDefinition>,

    pub enum_definitions: Vec<EnumDefinition>,
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum EnumDefinition {
    StringEnum(StringEnumDefinition),

    IntEnum(IntEnumDefinition),
}

#[derive(Debug)]
pub struct StringEnumDefinition {
    pub name: String,

    pub doc: String,

    pub variants: Vec<StringEnumInner>,
}

#[derive(Debug)]
pub struct StringEnumInner {
    pub index: usize,

    pub name: String,

    pub doc: String,

    pub value: Option<String>,
}

impl From<StringEnumDefinition> for crate::intermediate::StringEnum {
    fn from(value: StringEnumDefinition) -> Self {
        let mut members = BTreeMap::new();

        for variant in value.variants.as_slice() {
            let name: Arc<str> = variant.name.clone().into();
            let index = variant.index;
            let doc = variant.doc.clone();
            members.insert(name.clone(), StringEnumVariant { name, doc, index });
        }

        Self {
            name: value.name,
            doc: value.doc,
            variants: members,
        }
    }
}

#[derive(Debug)]
pub struct IntEnumDefinition {
    pub name: String,

    pub start: i128,

    pub doc: String,

    pub variants: Vec<IntEnumInner>,
}

#[derive(Debug)]
pub struct IntEnumInner {
    pub index: usize,

    pub name: String,

    pub doc: String,

    pub value: Option<i128>,
}

impl From<IntEnumDefinition> for crate::intermediate::IntEnum {
    fn from(value: IntEnumDefinition) -> Self {
        let mut members = BTreeMap::new();

        for variant in value.variants {
            let name: Arc<str> = variant.name.clone().into();
            let value = variant.value;
            let doc = variant.doc;
            members.insert(
                name.clone(),
                IntEnumVariant {
                    index: variant.index,
                    name,
                    doc,
                    value,
                },
            );
        }

        Self {
            start: value.start,
            name: value.name,
            doc: value.doc,
            variants: members,
        }
    }
}

#[derive(Debug)]
pub struct HTTPDefinition {
    pub name: String,

    pub fields: Vec<HTTPProperty>,
}

#[derive(Debug)]
pub struct HTTPProperty {
    pub name: String,

    pub r#type: DataType,

    pub r#encoding: Option<TypeEncoding>,

    pub key: String,

    pub doc: String,

    pub escape: bool,
}

#[derive(Debug)]
pub struct JsonDefinition {
    pub name: String,

    pub doc: String,

    pub hash: Option<String>,

    pub fields: Vec<JsonField>,
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum ArraySeparator {
    /// Array separated by ','
    ///
    /// ## Example
    ///
    /// [i32] = "1,3,4,5,6"
    Comma,

    /// Array separated by '@'
    ///
    /// ## Example
    ///
    /// [i32] = "1@3@4@5@6"
    At,

    /// Array separated by '|'
    ///
    /// ## Example
    ///
    /// [i32] = "1|3|4|5|6"
    Colon,
}

impl FromStr for ArraySeparator {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "," => Ok(Self::Comma),
            "@" => Ok(Self::At),
            "|" => Ok(Self::Colon),
            _ => Err(
                "expected to find one of `,` (comma), `@` (at), or `|` (colon) as array separator",
            ),
        }
    }
}

impl Display for ArraySeparator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArraySeparator::Comma => write!(f, ","),
            ArraySeparator::At => write!(f, "@"),
            ArraySeparator::Colon => write!(f, "|"),
        }
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum DataType {
    I32 {
        encoding: TypeEncoding,
    },

    U32 {
        encoding: TypeEncoding,
    },

    I64 {
        encoding: TypeEncoding,
    },

    U64 {
        encoding: TypeEncoding,
    },

    F32 {
        encoding: TypeEncoding,
    },

    // NOTE(anri, 2026-01-11): a string-encoded 64-bit float does not exist yet.
    F64,

    Bool {
        encoding: TypeEncoding,
    },

    /// Date time formatted as '2026-11-01 03:43:04'.
    Datetime,

    /// UNIX timestamp.
    DatetimeUnix,

    String,

    /// An array typically represented as a string separated by a separator.
    StringArray {
        inner: Arc<DataType>,
        separator: ArraySeparator,
    },

    /// A normal array.
    Array(Arc<DataType>),

    /// Like `Array` but it only holds a single element.
    SingleElementArray(Arc<DataType>),

    /// Dictionary from one type to another.
    Map {
        key: Arc<DataType>,
        value: Arc<DataType>,
    },

    // Tuple(Vec<DataType>),
    Custom(String),
}

impl Display for DataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataType::I32 { encoding: _ } => write!(f, "i32"),
            DataType::U32 { encoding: _ } => write!(f, "u32"),
            DataType::I64 { encoding: _ } => write!(f, "i64"),
            DataType::U64 { encoding: _ } => write!(f, "u64"),
            DataType::F32 { encoding: _ } => write!(f, "f32"),
            DataType::F64 => write!(f, "f64"),
            DataType::Bool { encoding: _ } => write!(f, "bool"),
            DataType::Datetime => write!(f, "datetime"),
            DataType::DatetimeUnix => write!(f, "datetime-unix"),
            DataType::String => write!(f, "string"),
            DataType::Array(inner) => write!(f, "[{inner}]"),
            DataType::StringArray { inner, separator } => write!(f, "[{inner}{separator}]"),
            DataType::SingleElementArray(data_type) => write!(f, "[{data_type}]"),
            DataType::Map { key, value } => write!(f, "{key} => {value}"),
            DataType::Custom(s) => write!(f, "{s}"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TypeEncoding {
    String,
    Int,
}

impl FromStr for TypeEncoding {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "str" | "string" => Ok(Self::String),
            "int" => Ok(Self::Int),
            _ => Err(format!("unknown encoding type `{s}`")),
        }
    }
}

#[derive(Debug, Clone)]
pub enum JSONKey {
    String(String),
    UseUnderlying,
}

impl From<JSONKey> for intermediate::JSONKey {
    fn from(value: JSONKey) -> Self {
        match value {
            JSONKey::String(s) => Self::String(s),
            JSONKey::UseUnderlying => Self::UseUnderlying,
        }
    }
}

#[derive(Debug)]
pub struct JsonField {
    pub name: String,

    pub r#type: DataType,

    pub key: JSONKey,

    pub doc: String,

    pub escape: bool,

    pub optional: bool,
}
