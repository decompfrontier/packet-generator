//! Source code generation.
//!
//! Given an [IR representation](crate::intermediate::DefinitionRegistry),
//! this modules generates the source files for parsing Brave Frontier's
//! packets in a given language.

use std::{
    borrow::Cow,
    fmt::Debug,
    io::Write,
    path::{Path, PathBuf},
};

use atomicow::CowArc;
use petgraph::{algo::Cycle, graph::NodeIndex};

use crate::intermediate::{
    DataType, DefinitionRegistry, IntEnum, IntEnumVariant, Json, JsonField, StringEnum,
    StringEnumVariant,
};

mod cpp;
mod glaze;
mod utils;

#[derive(Debug, Clone)]
pub struct GeneratedSource {
    pub filename: PathBuf,
    pub content: String,
}

pub trait Generator {
    /// # Errors
    fn generate(
        &self,
        registry: &DefinitionRegistry,
        initial_filename: &str,
    ) -> Result<Vec<GeneratedSource>, GenerationError>;

    fn json_name<'a>(&'a self, definition: &'a Json) -> CowArc<'a, str> {
        CowArc::Borrowed(&definition.name)
    }

    fn json_field_name<'a>(&'a self, definition: &'a JsonField) -> CowArc<'a, str> {
        CowArc::Borrowed(&definition.name)
    }

    fn int_enum_name<'a>(&'a self, definition: &'a IntEnum) -> CowArc<'a, str> {
        CowArc::Borrowed(&definition.name)
    }

    fn int_enum_variant_name<'a>(&'a self, definition: &'a IntEnumVariant) -> CowArc<'a, str> {
        CowArc::Borrowed(&definition.name)
    }

    fn string_enum_name<'a>(&'a self, definition: &'a StringEnum) -> CowArc<'a, str> {
        CowArc::Borrowed(&definition.name)
    }

    fn string_enum_variant_name<'a>(
        &'a self,
        definition: &'a StringEnumVariant,
    ) -> CowArc<'a, str> {
        CowArc::Borrowed(&definition.name)
    }
}

pub trait WithAddons {
    fn add_addon<T>(&mut self, addon: T)
    where
        T: Addon<For = Self> + 'static,
        Self: Sized;
}

pub trait Addon: Debug {
    type For;

    fn preamble(&self, _registry: &DefinitionRegistry) -> Option<Cow<'static, str>> {
        None
    }

    fn content(
        &self,
        _registry: &DefinitionRegistry,
    ) -> Option<Result<Cow<'static, str>, GenerationError>> {
        None
    }

    fn postamble(&self, _registry: &DefinitionRegistry) -> Option<Cow<'static, str>> {
        None
    }
}

/// Error type concerning problem when generating source files.
#[derive(Debug, Clone, thiserror::Error)]
pub enum GenerationError {
    /// We needed to look into a
    /// [`Definition`](crate::intermediate::Definition),
    /// but the [`DefinitionRegistry`](crate::intermediate::DefinitionRegistry)
    /// expired in the meantime.
    ///
    /// Use this when converting [`std::sync::Weak`] into [`std::sync::Arc`]
    /// through [`Weak::upgrade`](std::sync::Weak::upgrade).
    #[error(
        "expired dependant type entry from type `{:#?}`; the registry may have been de-allocated",
        queried_from
    )]
    ExpiredRegistry {
        /// The [`DataType`] from which the query was made.
        queried_from: DataType,
    },

    /// The type is not present in the
    /// [`DefinitionRegistry`](crate::intermediate::DefinitionRegistry).
    #[error(
        "datatype `{queried_from:#?}` depended on type definition `{name}`, but the latter was not found"
    )]
    TypeNotFound {
        /// The type we were looking for.
        name: String,

        /// The type from which the query originated.
        queried_from: DataType,
    },

    /// There is a cycle between definitions in the
    /// [`DefinitionRegistry`](crate::intermediate::DefinitionRegistry).
    #[error("found a cycle in the definition registry: node #{0:?}")]
    CycleFound(Cycle<NodeIndex>),
}

/// Writes the generated sources to the files obtained from the generator,
/// while using `base_directory` as the base path.
///
/// # Errors
///
/// Returns an error if the
pub fn write_sources(
    base_directory: impl AsRef<Path>,
    sources: &[GeneratedSource],
) -> Result<(), miette::Error> {
    for source in sources {
        let output_file = base_directory.as_ref().join(&source.filename);

        if let Some(path) = output_file.parent() {
            std::fs::create_dir_all(path).map_err(|e| {
                miette::miette!(
                    "could not create directories for path `{}`: {e}",
                    path.display()
                )
            })?;
        }

        let mut file = std::fs::File::create(&output_file).map_err(|e| {
            miette::miette!(
                "failed to open output file `{}`: {e}",
                output_file.display()
            )
        })?;

        file.write_all(source.content.as_bytes()).map_err(|e| {
            miette::miette!("failed to write to file `{}`: {e}", output_file.display())
        })?;
    }

    Ok(())
}

pub use cpp::CxxGenerator;
pub use glaze::GlazeGenerator;
