use std::{
    collections::BTreeSet, fmt::Display, num::NonZeroUsize, path::PathBuf, str::FromStr, sync::Arc,
};

mod validator;

use miette::SourceSpan;
pub use validator::validate;

use crate::{
    intermediate::{IntEnumVariant, StringEnumVariant},
    kdl_parser::SourceInfo,
};

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
    pub index: usize,

    pub source_info: Arc<SourceInfo>,

    pub span: SourceSpan,

    pub name: String,

    pub doc: String,

    pub variants: Vec<StringEnumInner>,
}

#[derive(Debug)]
pub struct StringEnumInner {
    pub index: usize,

    pub source_info: Arc<SourceInfo>,
    pub span: SourceSpan,

    pub name: String,

    pub doc: String,

    pub value: String,
}

impl From<StringEnumDefinition> for crate::intermediate::StringEnum {
    fn from(value: StringEnumDefinition) -> Self {
        let mut members = BTreeSet::new();

        for variant in value.variants.as_slice() {
            let name: Arc<str> = variant.name.clone().into();
            let doc = variant.doc.clone();
            let val = variant.value.clone();
            members.insert(StringEnumVariant {
                name,
                doc,
                value: val,
                index: variant.index,
            });
        }

        Self {
            index: value.index,
            name: value.name,
            doc: value.doc,
            variants: members,
        }
    }
}

#[derive(Debug)]
pub struct IntEnumDefinition {
    pub index: usize,

    pub source_info: Arc<SourceInfo>,
    pub span: SourceSpan,

    pub name: String,

    pub start: i128,

    pub doc: String,

    pub variants: Vec<IntEnumInner>,
}

#[derive(Debug)]
pub struct IntEnumInner {
    pub index: usize,

    pub source_info: Arc<SourceInfo>,

    pub span: SourceSpan,

    pub name: String,

    pub doc: String,

    pub value: Option<i128>,
}

impl From<IntEnumDefinition> for crate::intermediate::IntEnum {
    fn from(value: IntEnumDefinition) -> Self {
        let mut members = BTreeSet::new();

        for variant in value.variants {
            let name: Arc<str> = variant.name.clone().into();
            let value = variant.value;
            let doc = variant.doc;
            members.insert(IntEnumVariant {
                index: variant.index,
                name,
                doc,
                value,
            });
        }

        Self {
            index: value.index,
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

    pub r#encoding: Option<IntLikeEncoding>,

    pub key: String,

    pub doc: String,

    pub escape: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum JsonEncoding {
    Json,
    String,
}

impl Display for JsonEncoding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Json => write!(f, "json"),
            Self::String => write!(f, "str"),
        }
    }
}

impl From<JsonEncoding> for crate::intermediate::JsonEncoding {
    fn from(value: JsonEncoding) -> Self {
        match value {
            JsonEncoding::Json => Self::Json,
            JsonEncoding::String => Self::String,
        }
    }
}

#[derive(Debug)]
pub struct JsonDefinition {
    pub index: usize,

    pub source_info: Arc<SourceInfo>,

    pub span: SourceSpan,

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
    Pipe,

    /// Array separated by ':'
    ///
    /// ## Example
    ///
    /// [i32] = "1:3:4:5:6"
    Colon,
}

impl FromStr for ArraySeparator {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "," => Ok(Self::Comma),
            "@" => Ok(Self::At),
            ":" => Ok(Self::Colon),
            "|" => Ok(Self::Pipe),
            _ => Err(
                "expected to find one of `,` (comma), `@` (at), `|` (pipe), or `:` (colon) as array separator",
            ),
        }
    }
}

impl Display for ArraySeparator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Comma => write!(f, ","),
            Self::At => write!(f, "@"),
            Self::Pipe => write!(f, "|"),
            Self::Colon => write!(f, ":"),
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum ArraySize {
    #[default]
    /// Array of unbounded elements.
    Dynamic,

    /// Array of exactly N elements.
    Fixed(NonZeroUsize),
}

impl Display for ArraySize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Dynamic => write!(f, "size(n)"),
            Self::Fixed(size) => write!(f, "size({size})"),
        }
    }
}

impl From<ArraySize> for crate::intermediate::ArraySize {
    fn from(value: ArraySize) -> Self {
        match value {
            ArraySize::Dynamic => Self::Dynamic,
            ArraySize::Fixed(n) => Self::Fixed(n),
        }
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum DataType {
    I32 {
        encoding: IntLikeEncoding,
    },

    U32 {
        encoding: IntLikeEncoding,
    },

    I64 {
        encoding: IntLikeEncoding,
    },

    U64 {
        encoding: IntLikeEncoding,
    },

    F32 {
        encoding: IntLikeEncoding,
    },

    // NOTE(anri, 2026-01-11): a string-encoded 64-bit float does not exist yet.
    F64,

    Bool {
        encoding: BoolEncoding,
    },

    /// Date time formatted as '2026-11-01 03:43:04'.
    Datetime,

    /// UNIX timestamp.
    DatetimeUnix,

    String,

    /// An array represented as a native array type.
    Array {
        size: ArraySize,
        inner: Arc<Self>,
    },

    /// An array typically represented as a string separated by a separator.
    StringArray {
        inner: Arc<Self>,
        separator: ArraySeparator,
        size: ArraySize,
    },

    /// Dictionary from one type to another.
    Map {
        key: Arc<Self>,
        value: Arc<Self>,
    },

    // Tuple(Vec<DataType>),
    Custom {
        name: String,
        encoding: JsonEncoding,
    },
}

impl Display for DataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::I32 { encoding: _ } => write!(f, "i32"),
            Self::U32 { encoding: _ } => write!(f, "u32"),
            Self::I64 { encoding: _ } => write!(f, "i64"),
            Self::U64 { encoding: _ } => write!(f, "u64"),
            Self::F32 { encoding: _ } => write!(f, "f32"),
            Self::F64 => write!(f, "f64"),
            Self::Bool { encoding: _ } => write!(f, "bool"),
            Self::Datetime => write!(f, "datetime"),
            Self::DatetimeUnix => write!(f, "datetime-unix"),
            Self::String => write!(f, "string"),
            Self::Array { size, inner } => write!(f, "[{inner}]::{size}"),
            Self::StringArray {
                inner,
                separator,
                size,
            } => write!(f, "[{inner}{separator}]::{size}"),
            Self::Map { key, value } => write!(f, "{key} => {value}"),
            Self::Custom { name, encoding } => write!(f, "{name}::{encoding}"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum IntLikeEncoding {
    String,
    Int,
}

impl FromStr for IntLikeEncoding {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "str" | "string" => Ok(Self::String),
            "int" => Ok(Self::Int),
            _ => Err(format!("unknown encoding type `{s}`")),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum BoolEncoding {
    String,
    Int,
    Bool,
}

impl FromStr for BoolEncoding {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "str" => Ok(Self::String),
            "int" => Ok(Self::Int),
            _ => Err(format!("unknown encoding type `{s}`")),
        }
    }
}

#[derive(Debug)]
pub struct JsonField {
    pub index: usize,

    pub source_info: Arc<SourceInfo>,

    pub span: SourceSpan,

    pub name: String,

    pub r#type: DataType,

    pub key: String,

    pub doc: String,

    pub escape: bool,

    pub optional: bool,
}
