use crate::kdl_parser::{
    Document,
    schema::{DataType, RawDocument},
};

use crate::kdl_parser::Diagnostic;

pub fn validate(document: RawDocument) -> Result<Document, Diagnostic> {
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
