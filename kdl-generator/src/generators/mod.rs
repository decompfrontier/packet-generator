//! Source code generation.
//!
//! Given an [IR representation](crate::intermediate::DefinitionRegistry),
//! this modules generates the source files for parsing Brave Frontier's
//! packets in a given language.

use std::{borrow::Cow, fmt::Debug};

use crate::intermediate::{DataType, DefinitionRegistry};

mod cpp;
mod glaze;
mod utils;

#[derive(Debug, Clone)]
pub struct GeneratedSource {
    pub filename: String,
    pub content: String,
}

pub trait Generator {
    /// # Errors
    fn generate(
        &self,
        registry: &DefinitionRegistry,
        initial_filename: &str,
    ) -> Result<GeneratedSource, GenerationError>;
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

pub use cpp::CxxGenerator;
pub use glaze::GlazeGenerator;
