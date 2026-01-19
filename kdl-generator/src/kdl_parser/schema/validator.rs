use crate::kdl_parser::{Document, schema::RawDocument};

use crate::kdl_parser::Diagnostic;

/// Validates a `RawDocument` to make it usable for the intermediate
/// representation.
///
/// # Errors
///
/// Returns `Err` if after this post-processing the `Document` is still not valid.
#[allow(clippy::result_large_err)]
pub const fn validate(document: RawDocument) -> Result<Document, Diagnostic> {
    // for struct_ in &document.data {
    //     for field in &struct_.fields {
    //         if matches!(
    //             field.r#type,
    //             DataType::I32
    //                 | DataType::U32
    //                 | DataType::I64
    //                 | DataType::U64
    //                 | DataType::F32
    //                 | DataType::F64
    //                 | DataType::Bool
    //         ) && field.encoding.is_none()
    //         {
    //             return Err(ValidationError::MissingEncoding {
    //                 struct_: struct_.name.clone(),
    //                 field: field.name.clone(),
    //                 recorded_type: field.r#type.clone(),
    //             });
    //         }
    //     }
    // }
    //
    // Ok(Document(document))
    Ok(Document(document))
}
