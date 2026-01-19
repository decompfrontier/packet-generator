//! Brave Frontier packet generator library.
//!
//! The end-goal of this library is to generate C++ source code for parsing
//! Brave Frontier's network packets.
//! The structure of the packets are defined over custom KDL files defined by
//! this library.

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
/// See [`ParsingError`].
pub fn parse_kdl<S: AsRef<str>>(
    document: S,
    filepath: &Path,
) -> Result<intermediate::DefinitionRegistry, ParsingError> {
    let raw_document = kdl_parser::raw_parse_kdl(document, filepath)?;

    let document = kdl_parser::validate(raw_document)?;

    Ok(kdl_parser::document_to_definitions(document))
}
