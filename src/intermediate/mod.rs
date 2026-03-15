//! Intermediate

use std::{
    borrow::Borrow,
    collections::{BTreeSet, HashMap},
    sync::Arc,
};

use core::num::NonZeroUsize;

use miette::SourceSpan;
use petgraph::{
    algo::toposort, graph::NodeIndex, prelude::StableDiGraph, stable_graph::NodeIndices,
};

use crate::kdl_parser::{Diagnostic, SourceInfo};

#[derive(Debug, Clone, thiserror::Error)]
pub enum RegistryError {
    #[error("the definition `{name}` refers to a type `{target}` that does not exist")]
    IncompleteDefinition {
        name: String,
        target: String,
        field_span: SourceSpan,
    },
}

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

type DefinitionRef = NodeIndex;

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
    pub encoding: JsonEncoding,
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
            encoding: JsonEncoding::Json,
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

fn extract_dependency_from_field(data_type: &DataType) -> Vec<DefinitionRef> {
    let mut ret = vec![];

    match data_type {
        DataType::Definition { definition, .. } => ret.push(*definition),

        DataType::Array { inner_type, .. } | DataType::StringArray { inner_type, .. } => {
            ret.append(&mut extract_dependency_from_field(inner_type.as_ref()));
        }

        DataType::Map { key, value } => {
            ret.append(&mut extract_dependency_from_field(key.as_ref()));
            ret.append(&mut extract_dependency_from_field(value.as_ref()));
        }

        _ => {}
    }

    ret
}

#[derive(Debug, Clone)]
pub struct DefinitionRegistry {
    definitions: StableDiGraph<Definition, ()>,
    names: HashMap<String, DefinitionRef>,
    _private: std::marker::PhantomData<()>,
}

impl DefinitionRegistry {
    #[must_use]
    pub fn get(&self, definition_ref: DefinitionRef) -> &Definition {
        &self.definitions[definition_ref]
    }

    pub fn find<S: AsRef<str>>(&self, name: S) -> Option<(&Definition, NodeIndex)> {
        self.names
            .get(name.as_ref())
            .map(|&idx| (&self.definitions[idx], idx))
    }

    #[must_use]
    pub fn find_weak<S: AsRef<str>>(&self, name: S) -> Option<NodeIndex> {
        self.names.get(name.as_ref()).copied()
    }

    #[must_use]
    pub fn all_definitions(&self) -> NodeIndices<'_, Definition> {
        self.definitions.node_indices()
    }

    /// # Errors
    ///
    /// Errors if the definitions have a cycle between each other.
    pub fn sorted_definitions(&self) -> Result<Vec<NodeIndex>, petgraph::algo::Cycle<NodeIndex>> {
        toposort(&self.definitions, None)
    }
}

#[derive(Debug, Clone, Default)]
pub struct PartialDefinitionRegistry {
    definitions: StableDiGraph<Definition, ()>,
    names: HashMap<String, DefinitionRef>,

    _private: std::marker::PhantomData<()>,
}

impl PartialDefinitionRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self {
            definitions: StableDiGraph::new(),
            names: HashMap::new(),
            _private: std::marker::PhantomData {},
        }
    }

    pub fn insert(&mut self, definition: Definition) -> DefinitionRef {
        #[allow(clippy::single_match_else, reason = "May add more cases in the future")]
        match definition {
            Definition::Json(ref json) => {
                let mut dependencies = vec![];

                for field in &json.fields {
                    dependencies.append(&mut extract_dependency_from_field(&field.type_));
                }

                let name = definition.name().clone();
                let this_def_idx = self.definitions.add_node(definition);
                self.names.insert(name, this_def_idx);
                for &dependency in &dependencies {
                    self.definitions.add_edge(dependency, this_def_idx, ());
                }

                this_def_idx
            }

            _ => {
                let name = definition.name().clone();
                let idx = self.definitions.add_node(definition);
                self.names.insert(name, idx);

                idx
            }
        }
    }

    /// Validates the current incomplete registry to build a
    /// [`DefinitionRegistry`].
    ///
    /// # Errors
    ///
    /// Return `Err` if the definitions reference non-existing definitions.
    #[allow(clippy::result_large_err, reason = "We can take the performance hit.")]
    pub fn finalize(mut self) -> Result<DefinitionRegistry, Diagnostic> {
        let all_nodes: Vec<_> = self.definitions.node_indices().collect();
        let mut missing_edges = vec![];

        for node_idx in &all_nodes {
            let node = &mut self.definitions[*node_idx];

            if let Definition::Json(json) = node {
                let fields = json
                    .fields
                    .iter()
                    .map(|field| -> Result<_, Diagnostic> {
                        let mut f = field.clone();

                        if let DataType::Unknown { encoding, name } = &field.type_ {
                            let idx = self.names.get(name).ok_or_else(|| Diagnostic {
                                message: format!("could not find definition `{name}`"),
                                severity: miette::Severity::Error,
                                source_info: json.source.clone(),
                                span: field.span,
                                help: None,
                                label: None,
                                related: vec![Diagnostic {
                                    message: format!(
                                        "the definition {name} is used inside field `{}::{}`",
                                        json.name, field.name
                                    ),
                                    severity: miette::Severity::Advice,
                                    source_info: json.source.clone(),
                                    span: field.span,
                                    help: None,
                                    label: None,
                                    related: vec![],
                                }],
                            })?;

                            f.type_ = DataType::Definition {
                                encoding: *encoding,
                                definition: *idx,
                            };

                            let Some(target_definition) = self.names.get(&json.name) else {
                                return Err(Diagnostic {
                                    message: format!(
                                        "could not find definition `{}`, which is myself!",
                                        json.name
                                    ),
                                    severity: miette::Severity::Error,
                                    source_info: json.source.clone(),
                                    span: json.span,
                                    help: None,
                                    label: None,
                                    related: vec![],
                                });
                            };

                            missing_edges.push((idx, target_definition));
                        }

                        Ok(f)
                    })
                    .collect::<Result<_, Diagnostic>>();

                json.fields = fields?;
            }
        }

        for (source, destination) in &missing_edges {
            self.definitions.add_edge(**source, **destination, ());
        }

        Ok(DefinitionRegistry {
            definitions: self.definitions,
            names: self.names,
            _private: std::marker::PhantomData {},
        })
    }

    // #[must_use]
    // fn get(&self, definition_ref: DefinitionRef) -> &Definition {
    //     &self.definitions[definition_ref]
    // }
    //
    // fn find<S: AsRef<str>>(&self, name: S) -> Option<(&Definition, NodeIndex)> {
    //     self.names
    //         .get(name.as_ref())
    //         .map(|&idx| (&self.definitions[idx], idx))
    // }

    #[must_use]
    pub fn find_weak<S: AsRef<str>>(&self, name: S) -> Option<NodeIndex> {
        self.names.get(name.as_ref()).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn registry_can_handle_circular_definitions() {
        let mut definitions = PartialDefinitionRegistry::new();

        let source = Arc::new(SourceInfo {
            name: String::from("test.kdl"),
            source_code: String::from("json {{}}"),
        });

        {
            let field = JsonField {
                index: 0,
                name: "bar".into(),
                key: String::from("bar"),
                type_: DataType::String,
                optional: false,
                doc: String::from("some documentation"),
                span: SourceSpan::from((0, 0)),
            };

            let mut s = Json::new(
                String::from("Foo"),
                0,
                Some(String::from("avdsfdsf")),
                String::from("some documentation"),
                source.clone(),
                SourceSpan::from((0, 0)),
            );
            s.add_field(field);

            definitions.insert(Definition::Json(s));
        };

        {
            let foo_struct = definitions
                .find_weak("Foo")
                .expect("Foo was inserted above.");

            let field = JsonField {
                index: 0,
                name: "has_foo".into(),
                key: String::from("bar"),
                type_: DataType::Definition {
                    definition: foo_struct,
                    encoding: JsonEncoding::Json,
                },
                optional: false,
                doc: String::from("some documentation"),
                span: SourceSpan::from((0, 0)),
            };

            let mut s = Json::new(
                String::from("Bar"),
                1,
                Some(String::from("avfdsfdsf")),
                String::from("some documentation"),
                source,
                SourceSpan::from((0, 0)),
            );
            s.add_field(field);

            definitions.insert(Definition::Json(s));
        };

        let definitions = definitions.finalize().expect("should resolve cycles?");

        definitions.find("Bar").expect("Bar was inserted above.");
    }
}
