use crate::kdl_parser::{Document, schema::RawDocument};

use crate::kdl_parser::Diagnostic;

#[expect(clippy::result_large_err, reason = "A Diagnostic is huge.")]
#[inline]
pub const fn validate(document: RawDocument) -> Result<Document, Diagnostic> {
    Ok(Document(document))
}
