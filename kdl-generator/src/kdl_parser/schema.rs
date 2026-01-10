use facet::Facet;
use facet_kdl as kdl;
use std::{collections::HashMap, fmt::Display, sync::Arc};

use crate::{
    intermediate::{IntEnumVariant, StringEnumVariant},
    kdl_parser::type_parser,
};

#[derive(Debug, Facet)]
pub struct RawDocument {
    #[facet(kdl::children)]
    pub data: Vec<DataDefinition>,

    #[facet(kdl::children)]
    pub json: Vec<JsonDefinition>,

    #[facet(kdl::children)]
    pub enums: Vec<EnumDefinition>,
}

#[derive(Debug, Facet)]
#[repr(C)]
#[allow(dead_code)]
pub enum EnumDefinition {
    #[facet(rename = "enum")]
    StringEnum(StringEnumDefinition),

    #[facet(rename = "ienum")]
    IntEnum(IntEnumDefinition),
}

#[derive(Debug, Facet)]
pub struct StringEnumDefinition {
    #[facet(kdl::argument)]
    pub name: String,

    #[facet(kdl::child)]
    pub doc: String,

    #[facet(kdl::children)]
    pub variants: Vec<StringEnumInner>,
}

#[derive(Debug, Facet)]
pub struct StringEnumInner {
    #[facet(kdl::argument)]
    pub name: String,

    #[facet(kdl::child)]
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

#[derive(Debug, Facet)]
pub struct IntEnumDefinition {
    #[facet(kdl::argument)]
    pub name: String,

    #[facet(kdl::property, default = 0)]
    pub start: isize,

    #[facet(kdl::child)]
    pub doc: String,

    #[facet(kdl::children)]
    pub variants: Vec<IntEnumInner>,
}

#[derive(Debug, Facet)]
pub struct IntEnumInner {
    #[facet(kdl::argument)]
    pub name: String,

    #[facet(kdl::child)]
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

#[derive(Debug, Facet)]
pub struct JsonDefinition {
    #[facet(kdl::argument)]
    pub name: String,

    #[facet(kdl::children)]
    pub fields: Vec<JsonProperty>,
}

#[derive(Debug, Facet)]
pub struct JsonProperty {
    #[facet(kdl::argument)]
    pub name: String,

    #[facet(kdl::property, proxy=DataTypeProxy)]
    pub r#type: DataType,

    #[facet(kdl::property)]
    pub r#encoding: Option<TypeEncoding>,

    #[facet(kdl::child)]
    pub key: String,

    #[facet(kdl::child)]
    pub doc: String,

    #[facet(kdl::child, default = false)]
    pub escape: bool,
}

#[derive(Debug, Facet)]
pub struct DataDefinition {
    #[facet(kdl::argument)]
    pub name: String,

    #[facet(kdl::child)]
    pub hash: String,

    #[facet(kdl::children)]
    pub fields: Vec<DataProperty>,
}

#[derive(Debug, Clone, Copy, Facet)]
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

    /// Array separated by ':'
    ///
    /// ## Example
    ///
    /// [i32] = "1:3:4:5:6"
    Colon,
}

impl Display for ArraySeparator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArraySeparator::Comma => write!(f, ","),
            ArraySeparator::At => write!(f, "@"),
            ArraySeparator::Colon => write!(f, ":"),
        }
    }
}

#[derive(Debug, Clone, Facet)]
#[repr(C)]
#[allow(dead_code)]
pub enum DataType {
    I32,
    U32,
    I64,
    U64,
    F32,
    F64,
    Bool,
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
            DataType::I32 => write!(f, "i32"),
            DataType::U32 => write!(f, "u32"),
            DataType::I64 => write!(f, "i64"),
            DataType::U64 => write!(f, "u64"),
            DataType::F32 => write!(f, "f32"),
            DataType::F64 => write!(f, "f64"),
            DataType::Bool => write!(f, "bool"),
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

#[derive(Debug, Facet)]
#[facet(transparent)]
struct DataTypeProxy(String);

impl TryFrom<DataTypeProxy> for DataType {
    type Error = String;

    fn try_from(value: DataTypeProxy) -> std::result::Result<Self, Self::Error> {
        type_parser::generic_parse(&value.0)
    }
}

#[expect(
    clippy::infallible_try_from,
    reason = "facet forces us to use `Infallible`."
)]
impl TryFrom<&DataType> for DataTypeProxy {
    type Error = std::convert::Infallible;

    fn try_from(_value: &DataType) -> std::result::Result<Self, Self::Error> {
        todo!("Needed for serialization, which we don't do at the moment.")
    }
}

#[derive(Debug, Clone, Copy, Facet)]
#[repr(C)]
pub enum TypeEncoding {
    #[facet(rename = "str")]
    String,

    #[facet(rename = "int")]
    Int,
}

#[derive(Debug, Facet)]
pub struct DataProperty {
    #[facet(kdl::argument)]
    pub name: String,

    #[facet(kdl::property, proxy=DataTypeProxy)]
    pub r#type: DataType,

    #[facet(kdl::property)]
    pub r#encoding: Option<TypeEncoding>,

    #[facet(kdl::child)]
    pub r#hash: String,

    #[facet(kdl::child)]
    pub doc: String,

    #[facet(kdl::child, default = false)]
    pub escape: bool,
}
