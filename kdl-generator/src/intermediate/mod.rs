//! Intermediate

use std::{
    borrow::Borrow,
    collections::{BTreeSet, HashMap},
    sync::Arc,
};

use petgraph::{
    algo::toposort, graph::NodeIndex, prelude::StableDiGraph, stable_graph::NodeIndices,
};

use crate::kdl_parser::Diagnostic;

#[derive(Debug, Clone, thiserror::Error)]
pub enum RegistryError {
    #[error("the definition `{name}` refers to a type `{target}` that does not exist")]
    IncompleteDefinition { name: String, target: String },
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

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Json {
    pub index: usize,
    pub name: String,
    pub hash_name: Option<String>,
    pub fields: BTreeSet<JsonField>,
    pub doc: String,
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
    pub const fn new(name: String, index: usize, hash_name: Option<String>, doc: String) -> Self {
        Self {
            index,
            name,
            hash_name,
            fields: BTreeSet::new(),
            doc,
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

    /// Array separated by ':'
    ///
    /// ## Example
    ///
    /// [i32] = "1:3:4:5:6"
    Colon,
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
    },

    Array {
        inner_type: Arc<Self>,
    },

    SingleElementArray {
        inner_type: Arc<Self>,
    },

    Definition(DefinitionRef),

    Unknown(String),
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

    /// Validates the current incomplete registry a builds a [`DefinitionRegistry`].
    ///
    /// # Errors
    ///
    /// Return `Err` if the definitions reference non-existing definitions.
    ///
    /// # Panics
    ///
    /// FIXME(anri): currently panics on the error conditions above.
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
                    .map(|field| -> Result<_, RegistryError> {
                        let mut f = field.clone();

                        if let DataType::Unknown(name) = &field.type_ {
                            let idx = self.names.get(name).ok_or_else(|| {
                                RegistryError::IncompleteDefinition {
                                    name: format!("{}::{}", json.name, f.name),
                                    target: name.clone(),
                                }
                            })?;

                            f.type_ = DataType::Definition(*idx);

                            missing_edges
                                .push((idx, self.names.get(&json.name).expect("this is myself")));
                        }

                        Ok(f)
                    })
                    .collect::<Result<_, RegistryError>>()
                    .expect("todo: remove this panic on favor of Result");

                json.fields = fields;
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

    pub fn insert(&mut self, definition: Definition) -> DefinitionRef {
        #[allow(clippy::single_match_else, reason = "May add more cases in the future")]
        match definition {
            Definition::Json(ref json) => {
                let dependencies: Vec<_> = json
                    .fields
                    .iter()
                    .filter_map(|field| match field.type_ {
                        DataType::Definition(def_idx) => Some(def_idx),

                        _ => None,
                    })
                    .collect();

                let name = definition.name().clone();
                let idx = self.definitions.add_node(definition);
                self.names.insert(name, idx);
                for &dependency in &dependencies {
                    self.definitions.add_edge(dependency, idx, ());
                }

                idx
            }

            _ => {
                let name = definition.name().clone();
                let idx = self.definitions.add_node(definition);
                self.names.insert(name, idx);

                idx
            }
        }
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

        {
            let field = JsonField {
                index: 0,
                name: "bar".into(),
                key: String::from("bar"),
                type_: DataType::String,
                optional: false,
                doc: String::from("some documentation"),
            };

            let mut s = Json::new(
                String::from("Foo"),
                0,
                Some(String::from("avdsfdsf")),
                String::from("some documentation"),
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
                type_: DataType::Definition(foo_struct),
                optional: false,
                doc: String::from("some documentation"),
            };

            let mut s = Json::new(
                String::from("Bar"),
                1,
                Some(String::from("avfdsfdsf")),
                String::from("some documentation"),
            );
            s.add_field(field);

            definitions.insert(Definition::Json(s));
        };

        let definitions = definitions.finalize().expect("should resolve cycles?");

        definitions.find("Bar").expect("Bar was inserted above.");
    }
}
