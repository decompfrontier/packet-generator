use std::{borrow::Borrow, collections::BTreeSet, num::NonZeroUsize, sync::Arc};

use miette::SourceSpan;
use petgraph::graph::NodeIndex;

use crate::kdl_parser::SourceInfo;

/// A weak reference to a [`Definition`]. Can be seen as being equivalent to
/// [`std::sync::Weak`].
///
/// A instance of this type is completely useless by itself and must be used
/// in the context of a [`DefinitionRegistry`](super::DefinitionRegistry).
pub type DefinitionRef = NodeIndex;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Encoding {
    String,
    Int,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum BoolEncoding {
    String,
    Int,
    Bool,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Definition {
    Json(Json),
    IntEnum(IntEnum),
    StringEnum(StringEnum),
}

impl Definition {
    #[must_use]
    pub const fn name(&self) -> &String {
        match self {
            Self::Json(json) => &json.name,
            Self::IntEnum(int_enum) => &int_enum.name,
            Self::StringEnum(string_enum) => &string_enum.name,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum JsonEncoding {
    Json,
    String,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Json {
    pub index: usize,
    pub name: String,
    pub hash_name: Option<String>,
    pub fields: BTreeSet<JsonField>,
    pub doc: String,
    pub source: Arc<SourceInfo>,
    pub span: SourceSpan,
}

impl Borrow<str> for Json {
    fn borrow(&self) -> &str {
        &self.name
    }
}

#[derive(Clone, Debug)]
pub struct JsonField {
    pub index: usize,
    pub name: Arc<str>,
    pub key: String,
    pub type_: DataType,
    pub optional: bool,
    pub doc: String,
    pub span: SourceSpan,
}

impl Borrow<str> for JsonField {
    fn borrow(&self) -> &str {
        &self.name
    }
}

impl PartialEq for JsonField {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
            && self.name == other.name
            && self.key == other.key
            && self.optional == other.optional
            && self.doc == other.doc
    }
}

impl Eq for JsonField {}

impl PartialOrd for JsonField {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for JsonField {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.index.cmp(&other.index) {
            core::cmp::Ordering::Equal => {}
            ord => return ord,
        }
        match self.name.cmp(&other.name) {
            core::cmp::Ordering::Equal => {}
            ord => return ord,
        }
        match self.key.cmp(&other.key) {
            core::cmp::Ordering::Equal => {}
            ord => return ord,
        }
        self.optional.cmp(&other.optional)
    }
}

impl Json {
    #[must_use]
    pub const fn new(
        name: String,
        index: usize,
        hash_name: Option<String>,
        doc: String,
        defined_in_source: Arc<SourceInfo>,
        defined_in_span: SourceSpan,
    ) -> Self {
        Self {
            index,
            name,
            hash_name,
            fields: BTreeSet::new(),
            doc,
            source: defined_in_source,
            span: defined_in_span,
        }
    }

    pub fn add_field(&mut self, field: JsonField) -> bool {
        self.fields.insert(field)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct StringEnum {
    pub index: usize,
    pub name: String,
    pub doc: String,
    pub variants: BTreeSet<StringEnumVariant>,
}

#[derive(Clone, Debug, Eq, PartialOrd, Ord)]
pub struct StringEnumVariant {
    pub index: usize,
    pub name: Arc<str>,
    pub doc: String,
    pub value: String,
}

impl Borrow<str> for StringEnumVariant {
    fn borrow(&self) -> &str {
        &self.name
    }
}

impl PartialEq for StringEnumVariant {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

impl StringEnum {
    #[must_use]
    pub const fn new(name: String, index: usize, doc: String) -> Self {
        Self {
            index,
            name,
            doc,
            variants: BTreeSet::new(),
        }
    }

    pub fn add_variant(&mut self, field: StringEnumVariant) -> bool {
        self.variants.insert(field)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct IntEnum {
    pub index: usize,
    pub name: String,
    pub start: i128,
    pub doc: String,
    pub variants: BTreeSet<IntEnumVariant>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct IntEnumVariant {
    pub index: usize,
    pub name: Arc<str>,
    pub doc: String,
    pub value: Option<i128>,
}

impl Borrow<str> for IntEnumVariant {
    fn borrow(&self) -> &str {
        &self.name
    }
}

impl IntEnum {
    #[must_use]
    pub const fn new(name: String, index: usize, doc: String, start: i128) -> Self {
        Self {
            index,
            name,
            start,
            doc,
            variants: BTreeSet::new(),
        }
    }

    pub fn add_variant(&mut self, field: IntEnumVariant) -> bool {
        self.variants.insert(field)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum ArraySize {
    #[default]
    /// Unbounded array size.
    Dynamic,

    /// Fixed array size.
    Fixed(NonZeroUsize),
}

#[derive(Clone, Debug)]
pub enum DataType {
    I32 {
        encoding: Encoding,
    },

    U32 {
        encoding: Encoding,
    },

    I64 {
        encoding: Encoding,
    },

    U64 {
        encoding: Encoding,
    },

    F32 {
        encoding: Encoding,
    },

    F64,

    Bool {
        encoding: BoolEncoding,
    },

    String,

    Datetime,

    DatetimeUnix,

    Map {
        key: Arc<Self>,
        value: Arc<Self>,
    },

    StringArray {
        inner_type: Arc<Self>,
        separator: ArraySeparator,
        size: ArraySize,
    },

    Array {
        inner_type: Arc<Self>,
        size: ArraySize,
    },

    Definition {
        encoding: JsonEncoding,
        definition: DefinitionRef,
    },

    Unknown {
        encoding: JsonEncoding,
        name: String,
    },
}
