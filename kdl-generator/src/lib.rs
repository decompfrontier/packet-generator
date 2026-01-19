pub mod generators;
pub mod intermediate;
pub mod kdl_parser;

use std::path::Path;

use crate::kdl_parser::ParsingError;

/// Parses a KDL file into a registry of definitions that can be used for
/// generating source code.
///
/// # Errors
///
/// Will return `Err` if it was not possible to parse the file in `document`
/// and its includes.
pub fn parse_kdl<S: AsRef<str>>(
    document: S,
    filepath: &Path,
) -> Result<intermediate::DefinitionRegistry, ParsingError> {
    let raw_document = kdl_parser::raw_parse_kdl(document, filepath)?;

    let document = kdl_parser::validate(raw_document)?;

    Ok(kdl_parser::document_to_definitions(document))
}
