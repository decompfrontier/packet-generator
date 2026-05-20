//! Brave Frontier packet generator library.
//!
//! The end-goal of this library is to generate C++ source code for parsing
//! Brave Frontier's network packets.
//! The structure of the packets are defined over custom KDL files defined by
//! this library.

#![forbid(clippy::expect_used, clippy::unwrap_used)]

pub mod generators;
pub mod intermediate;
pub mod kdl_parser;

pub mod vfs;

use crate::{
    kdl_parser::{ParserOpts, ParsingError, ParsingWarnings, UnparsedKdl},
    vfs::Vfs,
};

/// Parses a KDL file into a registry of definitions that can be used for
/// generating source code.
///
/// # Example
///
/// ```rust
/// # use std::path::PathBuf;
/// use packet_generator::kdl_parser::{ParserOpts, UnparsedKdl};
///
/// # fn main() {
/// let doc = r#"
///     json Foo {
///         doc "Is a foo!"
///         field bar type="str" {
///             key "bar"
///             doc "A bar inside a Foo"
///         }
///     }
/// "#;
///
/// let path = PathBuf::from("foo.kdl");
///
/// let unparsed_kdl = UnparsedKdl::new(&doc, &path);
///
/// let opts = ParserOpts::default();
/// # let filemap = packet_generator::vfs::InMemoryFS::new();
/// # let opts = ParserOpts::new(filemap);
///
/// match packet_generator::parse_kdl(&[unparsed_kdl], &opts) {
///     Ok((registry, _warnings)) => {
///         println!("{:#?}", registry.find("Foo"));
///     }
///
///     Err(e) => println!("Error: {e}"),
/// }
/// # }
/// ```
///
/// # Errors
///
/// Will return `Err` if it was not possible to parse the file in `document`
/// and its includes.
/// See [`ParsingError`].
pub fn parse_kdl<V: Vfs>(
    documents: &[UnparsedKdl],
    opts: &ParserOpts<V>,
) -> Result<(intermediate::DefinitionRegistry, ParsingWarnings), ParsingError> {
    let (raw_document, warnings) = kdl_parser::raw_parse_kdl(documents, opts)?;

    let document = raw_document.finalize()?;

    Ok((kdl_parser::document_to_definitions(document)?, warnings))
}
