pub mod generators;
pub mod intermediate;
pub mod kdl_parser;

use crate::kdl_parser::ParsingError;

pub fn parse_kdl<S: AsRef<str>>(
    document: S,
) -> Result<intermediate::DefinitionRegistry, ParsingError> {
    let raw_document = kdl_parser::raw_parse_kdl(document)?;

    let document = kdl_parser::validate(raw_document)?;

    Ok(kdl_parser::document_to_definitions(document))
}
