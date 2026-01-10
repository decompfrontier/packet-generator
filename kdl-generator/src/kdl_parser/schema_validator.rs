use crate::kdl_parser::schema::DataType;

use rootcause::prelude::*;

use super::{Document, RawDocument};

#[derive(Debug, Clone, thiserror::Error)]
pub enum ValidationError {
    #[error("missing `encoding` for field `{field}`")]
    MissingEncoding {
        struct_: String,
        field: String,
        recorded_type: DataType,
    },
}

impl miette::Diagnostic for ValidationError {
    fn code<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        match self {
            ValidationError::MissingEncoding {
                struct_,
                field,
                recorded_type,
            } => {
                let error_msg = format!(
                    r#"
data {struct_} {{
    // ...
    field {field} type={recorded_type} {{"#
                );

                Some(Box::new(error_msg))
            }
        }
    }

    fn severity(&self) -> Option<miette::Severity> {
        match self {
            Self::MissingEncoding { .. } => Some(miette::Severity::Error),
        }
    }

    fn help<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        match self {
            ValidationError::MissingEncoding { .. } => Some(Box::new(String::from(
                "Integer, float and bool types need to specify an encoding: either `encoding=int` or `encoding=str`.",
            ))),
        }
    }

    fn url<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        None
    }

    fn source_code(&self) -> Option<&dyn miette::SourceCode> {
        None
    }

    fn labels(&self) -> Option<Box<dyn Iterator<Item = miette::LabeledSpan> + '_>> {
        None
    }

    fn related<'a>(&'a self) -> Option<Box<dyn Iterator<Item = &'a dyn miette::Diagnostic> + 'a>> {
        None
    }

    fn diagnostic_source(&self) -> Option<&dyn miette::Diagnostic> {
        None
    }
}

pub fn validate(document: RawDocument) -> Result<Document, ValidationError> {
    for struct_ in &document.data {
        for field in &struct_.fields {
            if matches!(
                field.r#type,
                DataType::I32
                    | DataType::U32
                    | DataType::I64
                    | DataType::U64
                    | DataType::F32
                    | DataType::F64
                    | DataType::Bool
            ) && field.encoding.is_none()
            {
                return Err(ValidationError::MissingEncoding {
                    struct_: struct_.name.clone(),
                    field: field.name.clone(),
                    recorded_type: field.r#type.clone(),
                });
            }
        }
    }

    Ok(Document(document))
}
