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
    /// The index of the definition in the given [source code](Self::source).
    pub index: usize,

    /// The name of the JSON definition.
    pub name: String,

    /// The fields inside the JSON definition.
    pub fields: BTreeSet<JsonField>,

    /// The documentation attached to the JSON definition.
    pub doc: String,

    /// The source file from which this JSON comes from,
    pub source: Arc<SourceInfo>,

    /// The span, related to a [source code](Self::source), where this JSON
    /// is defined.
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
        doc: String,
        defined_in_source: Arc<SourceInfo>,
        defined_in_span: SourceSpan,
    ) -> Self {
        Self {
            index,
            name,
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

/// Dev-only module for easy `proptest` tests.
#[cfg(test)]
pub mod arbitrary {
    use super::*;
    use proptest::prelude::*;

    pub fn arbitrary_int_encoding() -> impl Strategy<Value = Encoding> {
        prop_oneof![Just(Encoding::Int), Just(Encoding::String)]
    }

    pub fn arbitrary_bool_encoding() -> impl Strategy<Value = BoolEncoding> {
        prop_oneof![
            Just(BoolEncoding::Bool),
            Just(BoolEncoding::Int),
            Just(BoolEncoding::String)
        ]
    }

    pub fn arbitrary_json_encoding() -> impl Strategy<Value = JsonEncoding> {
        prop_oneof![Just(JsonEncoding::Json), Just(JsonEncoding::String)]
    }

    pub fn arbitrary_array_size() -> impl Strategy<Value = ArraySize> {
        prop_oneof![
            Just(ArraySize::Dynamic),
            any::<NonZeroUsize>().prop_map(ArraySize::Fixed)
        ]
    }

    pub fn arbitrary_array_separator() -> impl Strategy<Value = ArraySeparator> {
        prop_oneof![
            Just(ArraySeparator::Comma),
            Just(ArraySeparator::Colon),
            Just(ArraySeparator::Pipe),
            Just(ArraySeparator::At),
        ]
    }

    prop_compose! {
        pub fn arbitrary_source_info()(name in ".*", source_code in ".*") -> SourceInfo {
            SourceInfo {
                name,
                source_code
            }
        }
    }

    prop_compose! {
        pub fn arbitrary_source_span()(start in 0..(usize::MAX - 100))(end in start..usize::MAX, start in Just(start)) -> SourceSpan {
            (start..end).into()
        }
    }

    prop_compose! {
        pub fn arbitrary_json()(
            name in "[a-zA-Z][a-zA-Z0-9]*",
            index in 0..usize::MAX,
            doc in ".*",
            defined_in_source in arbitrary_source_info(),
            defined_in_span in arbitrary_source_span()
        ) -> Json {
            Json::new(name, index, doc, Arc::new(defined_in_source), defined_in_span)
        }
    }

    prop_compose! {
        pub fn arbitrary_unknown_datatype()(
            name in ".*",
            encoding in arbitrary_json_encoding()
        ) -> DataType {
            DataType::Unknown { encoding, name }
        }
    }

    prop_compose! {
        pub fn arbitrary_definition_ref()(x in any::<usize>()) -> DefinitionRef {
            DefinitionRef::new(x)
        }
    }

    fn arbitrary_self_referential_datatype(
        inner: BoxedStrategy<DataType>,
    ) -> impl Strategy<Value = DataType> {
        prop_oneof![
            (inner.clone(), arbitrary_array_size()).prop_map(|(inner_type, size)| {
                DataType::Array {
                    inner_type: Arc::new(inner_type),
                    size,
                }
            }),
            (
                inner.clone(),
                arbitrary_array_separator(),
                arbitrary_array_size()
            )
                .prop_map(|(inner_type, separator, size)| {
                    DataType::StringArray {
                        inner_type: Arc::new(inner_type),
                        size,
                        separator,
                    }
                }),
            inner.prop_map(|inner_type| {
                DataType::Map {
                    key: Arc::new(inner_type.clone()),
                    value: Arc::new(inner_type),
                }
            }),
        ]
    }

    pub fn arbitrary_datatype() -> impl Strategy<Value = DataType> {
        let leaf = prop_oneof![
            arbitrary_int_encoding().prop_map(|encoding| DataType::I32 { encoding }),
            arbitrary_int_encoding().prop_map(|encoding| DataType::U32 { encoding }),
            arbitrary_int_encoding().prop_map(|encoding| DataType::I64 { encoding }),
            arbitrary_int_encoding().prop_map(|encoding| DataType::U64 { encoding }),
            arbitrary_int_encoding().prop_map(|encoding| DataType::F32 { encoding }),
            arbitrary_bool_encoding().prop_map(|encoding| DataType::Bool { encoding }),
            Just(DataType::F64),
            Just(DataType::String),
            Just(DataType::Datetime),
            Just(DataType::DatetimeUnix),
            (arbitrary_definition_ref(), arbitrary_json_encoding()).prop_map(
                |(definition, encoding)| DataType::Definition {
                    encoding,
                    definition
                }
            ),
            arbitrary_unknown_datatype()
        ];

        leaf.prop_recursive(8, 200, 10, arbitrary_self_referential_datatype)
    }

    pub fn arbitrary_primitive_datatype() -> impl Strategy<Value = DataType> {
        let leaf = prop_oneof![
            arbitrary_int_encoding().prop_map(|encoding| DataType::I32 { encoding }),
            arbitrary_int_encoding().prop_map(|encoding| DataType::U32 { encoding }),
            arbitrary_int_encoding().prop_map(|encoding| DataType::I64 { encoding }),
            arbitrary_int_encoding().prop_map(|encoding| DataType::U64 { encoding }),
            arbitrary_int_encoding().prop_map(|encoding| DataType::F32 { encoding }),
            arbitrary_bool_encoding().prop_map(|encoding| DataType::Bool { encoding }),
            Just(DataType::F64),
            Just(DataType::String),
            Just(DataType::Datetime),
            Just(DataType::DatetimeUnix),
        ];

        leaf.prop_recursive(8, 200, 10, arbitrary_self_referential_datatype)
    }

    pub fn arbitrary_non_primitive_datatype() -> impl Strategy<Value = DataType> {
        let leaf = prop_oneof![
            (arbitrary_definition_ref(), arbitrary_json_encoding()).prop_map(
                |(definition, encoding)| DataType::Definition {
                    encoding,
                    definition
                }
            ),
        ];

        leaf.prop_recursive(8, 200, 10, arbitrary_self_referential_datatype)
    }
}
