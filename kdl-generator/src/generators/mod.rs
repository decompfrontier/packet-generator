//! Source code generation.
//!
//! Given an [IR representation](crate::intermediate::DefinitionRegistry),
//! this modules generates the source files for parsing Brave Frontier's
//! packets in a given language.

use crate::intermediate::{DataType, DefinitionRegistry};

mod cpp;
mod glaze;

pub trait SecondaryGenerator {
    fn get_prefix(&self) -> String {
        String::new()
    }

    fn get_suffix(&self) -> String {
        String::new()
    }

    fn step(
        &self,
        registry: &DefinitionRegistry
    ) -> Result<String, Report<GenerationError>>;
}

pub trait PrimaryGenerator: SecondaryGenerator {
    fn get_output_file_name(&self, name: &str) -> String;
}

/// Error type concerning problem when generating source files.
#[derive(Debug, Clone, thiserror::Error)]
pub enum GenerationError {
    /// We needed to look into a [`Definition`](crate::intermediate::Definition),
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
}

use rootcause::Report;

pub use cpp::CxxGenerator;
pub use glaze::GlazeGenerator;

