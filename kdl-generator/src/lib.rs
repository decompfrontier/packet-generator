pub mod generators;
pub mod intermediate;
pub mod kdl_parser;

use rootcause::prelude::*;

use crate::kdl_parser::ValidationError;

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error(transparent)]
    ParsingError(#[from] facet_kdl::KdlDeserializeError),

    #[error("validation error")]
    ValidationError(#[from] ValidationError),
}

pub fn parse_kdl<S: AsRef<str>>(
    document: S,
) -> Result<intermediate::DefinitionRegistry, Report<Error>> {
    let document: kdl_parser::schema::RawDocument = facet_kdl::from_str(document.as_ref())
        .context_transform_nested(|e| Error::ParsingError(e))?;

    let document =
        kdl_parser::validate(document).context_transform_nested(|e| Error::ValidationError(e))?;

    Ok(kdl_parser::document_to_definitions(document))
}
