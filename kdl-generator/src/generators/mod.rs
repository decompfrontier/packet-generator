use crate::intermediate::DataType;

mod cpp;
mod glaze;

#[derive(Debug, Clone, thiserror::Error)]
pub enum GenerationError {
    #[error("whatever")]
    Whatever,

    #[error(
        "expired dependant type entry from type `{:#?}`; the registry may have been de-allocated",
        queried_from
    )]
    ExpiredRegistry { queried_from: DataType },

    #[error(
        "datatype `{queried_from:#?}` depended on type definition `{name}`, but the latter was not found"
    )]
    TypeNotFound {
        name: String,
        queried_from: DataType,
    },
}

// TODO(arves): Make a function that takes some enum about the json generator and export it there,
//              rather than doing this pub use stuff that I'm doing right now
pub use cpp::generate_cxx;
pub use glaze::generate_glaze;
