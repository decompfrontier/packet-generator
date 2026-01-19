//! Source code generation.
//!
//! Given an [IR representation](crate::intermediate::DefinitionRegistry),
//! this modules generates the source files for parsing Brave Frontier's
//! packets in a given language.

use crate::intermediate::DataType;

mod cpp;
mod glaze;

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

// TODO(arves): Make a function that takes some enum about the json generator and export it there,
//              rather than doing this pub use stuff that I'm doing right now
pub use cpp::generate_cxx;
pub use glaze::generate_glaze;
