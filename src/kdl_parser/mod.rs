// TODO(anri): rename the module.
//! Module for parsing the KDL files into [`Document`]s.

use crate::{intermediate::DefinitionRegistry, kdl_parser::schema::RawDocument, vfs::Vfs};

mod errors;
mod parser;
pub mod schema;
mod to_intermediate;

// Exports
pub use errors::{Diagnostic, ParsingError, ParsingWarnings, SourceInfo};
pub use parser::UnparsedKdl;
pub use parser::raw_parse_kdl;

/// A valid KDL document.
///
/// This is the only type that can be converted to the IR, later used by
/// the generator, by calling [`document_to_definitions`].
pub struct Document(RawDocument);

/// The trait parameter `V` specifies which [VFS](crate::vfs) to use.
#[derive(Debug, Clone)]
pub struct ParserOpts<V = crate::vfs::DefaultFS>
where
    V: Vfs,
{
    /// The [VFS](crate::vfs) to use.
    vfs: V,
}

impl Default for ParserOpts {
    fn default() -> Self {
        Self {
            vfs: crate::vfs::DefaultFS,
        }
    }
}

impl<V: Vfs> ParserOpts<V> {
    pub const fn new(vfs: V) -> Self {
        Self { vfs }
    }
}

#[must_use = "Converting a `Document` to the IR representation implies that you want to use the resulting registry."]
#[allow(clippy::result_large_err, reason = "We can take the performance hit.")]
/// Converts a [`Document`] into a [`DefinitionRegistry`] (IR), returning
/// diagnostics in the process.
///
/// This is the only entrypoint for converting the result of KDL's parser into
/// the intermediate representation used by a
/// [`Generator`](crate::generators::Generator).
///
/// # Errors
///
/// Returns [`Diagnostic`] on error if validation fails.
/// In particular, this step checks and resolves every usage of a
/// [`Definition`](crate::intermediate::schema::Definition):
/// a [`Diagnostic`] is returned if an unknown
/// [`Definition`](crate::intermediate::schema::Definition) is used.
pub fn document_to_definitions(document: Document) -> Result<DefinitionRegistry, Diagnostic> {
    let mut registry = DefinitionRegistry::new();

    to_intermediate::add_enum_definitions(&mut registry, document.0.enum_definitions);
    to_intermediate::add_json_definitions(&mut registry, document.0.json_definitions);

    registry.finalize()
}
