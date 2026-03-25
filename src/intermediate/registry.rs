use std::collections::HashMap;
use std::marker::PhantomData;

use super::schema::*;

use miette::SourceSpan;
use petgraph::algo::toposort;
use petgraph::prelude::*;
use petgraph::stable_graph::NodeIndices;

use crate::intermediate::registry::sealed::RegistryState;
use crate::kdl_parser::Diagnostic;

#[derive(Debug, Clone, thiserror::Error)]
pub enum RegistryError {
    #[error("the definition `{name}` refers to a type `{target}` that does not exist")]
    IncompleteDefinition {
        name: String,
        target: String,
        field_span: SourceSpan,
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

mod sealed {
    pub trait RegistryState {}
}

/// Marker type that indicated that the [`DefinitionRegistry`] is
/// incomplete/partial and may not be valid.
#[derive(Debug)]
pub struct Partial;

/// Marker type that indicates that the [`DefinitionRegistry`] _must_ be valid.
#[derive(Debug)]
pub struct Complete;

impl sealed::RegistryState for Partial {}

impl sealed::RegistryState for Complete {}

/// The Intermediate Representation consumed by a
/// [`Generator`](crate::generators::Generator).
/// A KDL [`Document`](crate::kdl_parser::Document) parses into this.
///
/// The registry keeps track of a graph of [`Definition`]s and their
/// dependencies.
///
/// This is a typestate struct which can only be initialized in the [`Partial`]
/// state through [`DefinitionRegistry::new`].
/// In the [`Partial`] state, it can be modified as one sees fit.
/// However, to be usable by any of the [Generators](crate::generators),
/// it must be validated through [`DefinitionRegistry::finalize`] which
/// returns [`DefinitionRegistry<Complete>`].
///
/// For ease of use the default state, when used in code, is [`Complete`].
#[derive(Debug, Clone)]
pub struct DefinitionRegistry<T: sealed::RegistryState = Complete> {
    definitions: StableDiGraph<Definition, ()>,
    names: HashMap<String, DefinitionRef>,
    _private: std::marker::PhantomData<T>,
}

impl Default for DefinitionRegistry<Partial> {
    fn default() -> Self {
        Self {
            definitions: StableDiGraph::default(),
            names: HashMap::default(),
            _private: PhantomData,
        }
    }
}

impl<T: RegistryState> DefinitionRegistry<T> {
    /// Searches a [`Definition`] by `name` and returns a (weak) reference to
    /// it.
    #[must_use]
    pub fn find_weak<S: AsRef<str>>(&self, name: S) -> Option<DefinitionRef> {
        self.names.get(name.as_ref()).copied()
    }
}

/// A partial [`DefinitionRegistry`] that can be mutated.
/// Even if this type is typically constructed from a
/// [`Document`](crate::kdl_parser::Document) (which by definition is valid,
/// since it is _not_ a
/// [`RawDocument`](crate::kdl_parser::schema::RawDocument)), it may not
/// _necessarily_ contain valid data and thus cannot be used as-is
/// by a [`Generator`](crate::generators::Generator).
///
/// # Invalid states
///
/// The most common case where this (partial) registry is invalid occurs when
/// a [`Definition`] references a non-existing [`Definition`].
/// We cannot, however, ensure that all references are valid during an
/// [`DefinitionRegistry::insert`] since a _referenced_ [`Definition`] may
/// be inserted later.
///
/// The method [`DefinitionRegistry::finalize`] resolves all references and
/// either returns a complete [`DefinitionRegistry<Complete>`] or errors.
impl DefinitionRegistry<Partial> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            definitions: StableDiGraph::new(),
            names: HashMap::new(),
            _private: std::marker::PhantomData {},
        }
    }

    /// Inserts a [`Definition`] into the registry and returns a
    /// [reference](DefinitionRef) to it.
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

    /// Validates the current partial registry to build a
    /// complete [`DefinitionRegistry<Complete>`](DefinitionRegistry<Complete>).
    ///
    /// # Errors
    ///
    /// Return `Err` if the [`Definition`]s reference non-existing
    /// [`Definition`]s.
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
}

/// A read-only _always valid_ registry, that can be used to generate
/// source code through a [`Generator`](crate::generators::Generator).
///
/// The only way to construct this type is from
/// [`DefinitionRegistry<Partial>::finalize`].
///
///
/// # Validity guarantees
///
/// Since the complete registry is _immutable_, the validity of the references
/// inside the [`Definition`]s rests on the correctness of
/// [`DefinitionRegistry<Partial>::finalize`].
impl DefinitionRegistry<Complete> {
    /// Directly obtains a strong reference to a [`Definition`]
    /// from a (weak) reference [`DefinitionRef`].
    #[must_use]
    pub fn get(&self, definition_ref: DefinitionRef) -> &Definition {
        &self.definitions[definition_ref]
    }

    /// Searches a [`Definition`] by `name` and returns both a (strong)
    /// reference to it (`&Definition`) and a (weak) reference.
    pub fn find<S: AsRef<str>>(&self, name: S) -> Option<(&Definition, DefinitionRef)> {
        self.names
            .get(name.as_ref())
            .map(|&idx| (&self.definitions[idx], idx))
    }

    /// Returns an iterator over all definitions.
    #[must_use]
    pub fn all_definitions(&self) -> NodeIndices<'_, Definition> {
        self.definitions.node_indices()
    }

    /// Returns a topological sort of all the [`Definition`]s.
    ///
    /// Mainly useful for programming languages where the declaration order
    /// matters (for example, C and C++).
    ///
    /// # Errors
    ///
    /// Errors if the definitions have a cycle between each other;
    /// that is, if the internal dependency graph _is not_ a
    /// [Directed Acyclic Graph](https://en.wikipedia.org/wiki/Directed_acyclic_graph)).
    pub fn sorted_definitions(&self) -> Result<Vec<NodeIndex>, petgraph::algo::Cycle<NodeIndex>> {
        toposort(&self.definitions, None)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::kdl_parser::SourceInfo;

    use super::*;

    #[test]
    pub fn registry_can_handle_circular_definitions() {
        let mut definitions = DefinitionRegistry::new();

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
