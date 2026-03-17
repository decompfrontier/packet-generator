use crate::kdl_parser::{Document, schema::RawDocument};

use crate::kdl_parser::Diagnostic;

#[allow(clippy::result_large_err)]
#[inline]
pub const fn validate(document: RawDocument) -> Result<Document, Diagnostic> {
    Ok(Document(document))
}
