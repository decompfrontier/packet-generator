use std::{collections::HashMap, fmt::Display, sync::Arc};

mod validator;

pub use validator::validate;

use crate::intermediate::{IntEnumVariant, StringEnumVariant};

#[derive(Debug)]
pub struct RawDocument {
    pub data: Vec<DataDefinition>,

    pub json: Vec<JsonDefinition>,

    pub enums: Vec<EnumDefinition>,
}

#[derive(Debug)]
#[repr(C)]
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
    pub name: String,

    pub doc: String,
}

impl From<StringEnumDefinition> for crate::intermediate::StringEnum {
    fn from(value: StringEnumDefinition) -> Self {
        let mut members = HashMap::new();

        for variant in value.variants.as_slice() {
            let name: Arc<str> = variant.name.clone().into();
            members.insert(name.clone(), StringEnumVariant { name });
        }

        Self {
            name: value.name,
            variants: members,
        }
    }
}

#[derive(Debug)]
pub struct IntEnumDefinition {
    pub name: String,

    pub start: isize,

    pub doc: String,

    pub variants: Vec<IntEnumInner>,
}

#[derive(Debug)]
pub struct IntEnumInner {
    pub name: String,

    pub doc: String,
}

impl From<IntEnumDefinition> for crate::intermediate::IntEnum {
    fn from(value: IntEnumDefinition) -> Self {
        let mut members = HashMap::new();

        for variant in value.variants {
            let name: Arc<str> = variant.name.clone().into();
            members.insert(name.clone(), IntEnumVariant { name });
        }

        Self {
            start: value.start,
            name: value.name,
            variants: members,
        }
    }
}

#[derive(Debug)]
pub struct JsonDefinition {
    pub name: String,

    pub fields: Vec<JsonProperty>,
}

#[derive(Debug)]
pub struct JsonProperty {
    pub name: String,

    pub r#type: DataType,

    pub r#encoding: Option<TypeEncoding>,

    pub key: String,

    pub doc: String,

    pub escape: bool,
}

#[derive(Debug)]
pub struct DataDefinition {
    pub name: String,

    pub doc: String,

    pub hash: String,

    pub fields: Vec<DataProperty>,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
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
#[repr(C)]
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
    F64 {
        encoding: TypeEncoding,
    },
    Bool {
        encoding: TypeEncoding,
    },
    Datetime,
    String,

    /// A JSON object.
    Json,

    /// An array typically represented as a string separated by a separator.
    Array {
        inner: Arc<DataType>,
        separator: ArraySeparator,
    },

    /// An array represented as any arbitrary JSON object.
    JsonArray {
        type_hint: Option<Arc<DataType>>,
    },

    /// Models the case where there is an array of only one element.
    ///
    /// NOTE(anri): Maybe this can be avoided?
    SingleElementArray(Arc<DataType>),

    // Function {
    //     input: Arc<DataType>,
    //     output: Arc<DataType>,
    // },

    // Dictionary from one type to another.
    Map {
        key: Arc<DataType>,
        value: Arc<DataType>,
    },

    Tuple(Vec<DataType>),
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
            DataType::F64 { encoding: _ } => write!(f, "f64"),
            DataType::Bool { encoding: _ } => write!(f, "bool"),
            DataType::Datetime => write!(f, "datetime"),
            DataType::String => write!(f, "string"),
            DataType::Json => write!(f, "json"),
            DataType::Array { inner, separator } => write!(f, "[{inner}{separator}]"),
            DataType::JsonArray { type_hint: _ } => write!(f, "[json]"),
            DataType::SingleElementArray(data_type) => write!(f, "[{data_type}]"),
            DataType::Map { key, value } => write!(f, "{key} => {value}"),
            DataType::Tuple(_data_types) => todo!(),
            DataType::Custom(s) => write!(f, "{s}"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub enum TypeEncoding {
    String,

    Int,
}

#[derive(Debug)]
pub struct DataProperty {
    pub name: String,

    // pub r#type: DataType,
    pub r#type: DataType,

    // pub r#encoding: Option<TypeEncoding>,
    pub r#hash: String,

    pub doc: String,

    pub escape: bool,
}
