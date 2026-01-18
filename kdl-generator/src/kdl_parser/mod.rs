mod parser;
pub mod schema;

use std::{fmt::Display, path::PathBuf, sync::Arc};

use miette::{LabeledSpan, MietteSpanContents, Severity, SourceCode, SourceSpan};
pub use schema::validate;

use kdl::KdlError;

pub struct Document(RawDocument);

use crate::{intermediate::DefinitionRegistry, kdl_parser::schema::RawDocument};

pub use parser::raw_parse_kdl;

#[derive(Debug, Clone)]
pub struct SourceInfo {
    name: String,
    source_code: String,
}

impl SourceInfo {
    pub fn new(name: impl AsRef<str>, source_code: impl AsRef<str>) -> Self {
        Self {
            name: name.as_ref().into(),
            source_code: source_code.as_ref().into(),
        }
    }
}

impl SourceCode for SourceInfo {
    fn read_span<'a>(
        &'a self,
        span: &SourceSpan,
        context_lines_before: usize,
        context_lines_after: usize,
    ) -> Result<Box<dyn miette::SpanContents<'a> + 'a>, miette::MietteError> {
        let inner_span =
            self.source_code
                .read_span(span, context_lines_before, context_lines_after)?;

        Ok(Box::new(
            MietteSpanContents::new_named(
                self.name.to_string(),
                inner_span.data(),
                *inner_span.span(),
                inner_span.line(),
                inner_span.column(),
                inner_span.line_count(),
            )
            .with_language("kdl"),
        ))
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub struct Diagnostic {
    pub message: String,
    pub severity: Severity,
    pub source_info: Arc<SourceInfo>,
    pub span: SourceSpan,
    pub help: Option<String>,
    pub label: Option<String>,
    pub related: Vec<Diagnostic>,
}

impl Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = self.message.clone();
        write!(f, "{message}")
    }
}

impl miette::Diagnostic for Diagnostic {
    fn severity(&self) -> Option<miette::Severity> {
        Some(self.severity)
    }

    fn help<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        match &self.help {
            Some(x) => Some(Box::new(x)),
            None => None,
        }
    }

    fn source_code(&self) -> Option<&dyn miette::SourceCode> {
        Some(&self.source_info)
    }

    fn labels(&self) -> Option<Box<dyn Iterator<Item = miette::LabeledSpan> + '_>> {
        let label = self.label.clone().unwrap_or_else(|| "here".to_owned());

        let labeled_span = LabeledSpan::new_with_span(Some(label), self.span);

        Some(Box::new(std::iter::once(labeled_span)))
    }

    fn related<'a>(&'a self) -> Option<Box<dyn Iterator<Item = &'a dyn miette::Diagnostic> + 'a>> {
        if !self.related.is_empty() {
            Some(Box::new(
                self.related.iter().map(|d| d as &dyn miette::Diagnostic),
            ))
        } else {
            None
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ParsingError {
    #[error(transparent)]
    KdlError(#[from] KdlError),

    #[error("problems when parsing packet definition")]
    Diagnostics {
        source_info: Arc<SourceInfo>,

        diagnostics: Vec<Diagnostic>,
    },

    #[error("failed to canonicalize file path \"{path}\"")]
    NoAbsoluteFilePath {
        path: PathBuf,

        source: std::io::Error,
    },

    #[error(transparent)]
    GenericIoError(#[from] std::io::Error),
}

impl From<Diagnostic> for ParsingError {
    fn from(diag: Diagnostic) -> Self {
        Self::Diagnostics {
            source_info: diag.source_info.clone(),
            diagnostics: vec![diag],
        }
    }
}

impl miette::Diagnostic for ParsingError {
    fn code<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        match self {
            ParsingError::KdlError(kdl_error) => kdl_error.code(),
            _ => None,
        }
    }

    fn severity(&self) -> Option<miette::Severity> {
        match self {
            ParsingError::KdlError(kdl_error) => kdl_error.severity(),
            _ => None,
        }
    }

    fn help<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        match self {
            ParsingError::KdlError(kdl_error) => kdl_error.help(),
            _ => None,
        }
    }

    fn url<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        match self {
            ParsingError::KdlError(kdl_error) => kdl_error.url(),
            _ => None,
        }
    }

    fn source_code(&self) -> Option<&dyn miette::SourceCode> {
        match self {
            ParsingError::KdlError(kdl_error) => kdl_error.source_code(),
            ParsingError::Diagnostics {
                source_info: source_code,
                ..
            } => Some(source_code),
            _ => None,
        }
    }

    fn labels(&self) -> Option<Box<dyn Iterator<Item = miette::LabeledSpan> + '_>> {
        match self {
            ParsingError::KdlError(kdl_error) => kdl_error.labels(),
            _ => None,
        }
    }

    fn related<'a>(&'a self) -> Option<Box<dyn Iterator<Item = &'a dyn miette::Diagnostic> + 'a>> {
        match self {
            ParsingError::KdlError(kdl_error) => kdl_error.related(),
            ParsingError::Diagnostics { diagnostics, .. } => Some(Box::new(
                diagnostics.iter().map(|d| d as &dyn miette::Diagnostic),
            )),

            _ => None,
        }
    }

    fn diagnostic_source(&self) -> Option<&dyn miette::Diagnostic> {
        match self {
            ParsingError::KdlError(kdl_error) => kdl_error.diagnostic_source(),
            _ => None,
        }
    }
}

// TODO(anri): move this module somewhere? Possibly try to find a better name.
mod document_to_intermediate {
    use std::sync::Arc;

    use super::schema::EnumDefinition;
    use crate::{
        intermediate::{
            self, BoolEncoding as IntermediateBoolEncoding, DataType as IntermediateDataType,
            Definition, DefinitionRegistry, Encoding, Json, JsonField,
        },
        kdl_parser::schema::{
            self, BoolEncoding, DataType as SchemaDataType, JsonDefinition as SchemaJsonDefinition,
            JsonField as SchemaJsonField, TypeEncoding,
        },
    };

    fn convert_datatype_recursive(
        type_: &schema::DataType,
        registry: &mut DefinitionRegistry,
    ) -> intermediate::DataType {
        match type_ {
            schema::DataType::I32 { encoding } => match encoding {
                TypeEncoding::String => IntermediateDataType::I32 {
                    encoding: Encoding::String,
                },
                TypeEncoding::Int => IntermediateDataType::I32 {
                    encoding: Encoding::Int,
                },
            },

            SchemaDataType::U32 { encoding } => match encoding {
                TypeEncoding::String => IntermediateDataType::U32 {
                    encoding: Encoding::String,
                },
                TypeEncoding::Int => IntermediateDataType::U32 {
                    encoding: Encoding::Int,
                },
            },

            SchemaDataType::I64 { encoding } => match encoding {
                TypeEncoding::String => IntermediateDataType::I64 {
                    encoding: Encoding::String,
                },
                TypeEncoding::Int => IntermediateDataType::I64 {
                    encoding: Encoding::Int,
                },
            },

            SchemaDataType::U64 { encoding } => match encoding {
                TypeEncoding::String => IntermediateDataType::U64 {
                    encoding: Encoding::String,
                },
                TypeEncoding::Int => IntermediateDataType::U64 {
                    encoding: Encoding::Int,
                },
            },

            SchemaDataType::F32 { encoding } => match encoding {
                TypeEncoding::String => IntermediateDataType::F32 {
                    encoding: Encoding::String,
                },
                TypeEncoding::Int => IntermediateDataType::F32 {
                    encoding: Encoding::Int,
                },
            },

            SchemaDataType::F64 => IntermediateDataType::F64,

            SchemaDataType::Bool { encoding } => match encoding {
                BoolEncoding::String => IntermediateDataType::Bool {
                    encoding: IntermediateBoolEncoding::String,
                },

                BoolEncoding::Int => IntermediateDataType::Bool {
                    encoding: IntermediateBoolEncoding::Int,
                },

                BoolEncoding::Bool => IntermediateDataType::Bool {
                    encoding: IntermediateBoolEncoding::Bool,
                },
            },

            SchemaDataType::Datetime => IntermediateDataType::Datetime,
            SchemaDataType::DatetimeUnix => IntermediateDataType::DatetimeUnix,

            SchemaDataType::String => IntermediateDataType::String,

            SchemaDataType::Array(datatype) => IntermediateDataType::Array {
                inner_type: Arc::new(convert_datatype_recursive(datatype, registry)),
            },

            SchemaDataType::StringArray { inner, separator } => {
                use crate::kdl_parser::schema;

                match separator {
                    schema::ArraySeparator::Comma => intermediate::DataType::StringArray {
                        separator: intermediate::ArraySeparator::Comma,
                        inner_type: Arc::new(convert_datatype_recursive(inner, registry)),
                    },

                    schema::ArraySeparator::At => intermediate::DataType::StringArray {
                        separator: intermediate::ArraySeparator::At,
                        inner_type: Arc::new(convert_datatype_recursive(inner, registry)),
                    },

                    schema::ArraySeparator::Colon => intermediate::DataType::StringArray {
                        separator: intermediate::ArraySeparator::Colon,
                        inner_type: Arc::new(convert_datatype_recursive(inner, registry)),
                    },
                }
            }

            SchemaDataType::SingleElementArray(data_type) => {
                intermediate::DataType::SingleElementArray {
                    inner_type: Arc::new(convert_datatype_recursive(data_type, registry)),
                }
            }

            SchemaDataType::Map { key, value } => intermediate::DataType::Map {
                key: Arc::new(convert_datatype_recursive(key, registry)),
                value: Arc::new(convert_datatype_recursive(value, registry)),
            },

            SchemaDataType::Custom(s) => {
                if let Some(def) = registry.find_weak(s) {
                    intermediate::DataType::Definition(def)
                } else {
                    intermediate::DataType::Unknown(s.to_owned())
                }
            }
        }
    }

    fn convert_json_datatype(
        schema_field: &SchemaJsonField,
        registry: &mut DefinitionRegistry,
    ) -> IntermediateDataType {
        convert_datatype_recursive(&schema_field.r#type, registry)
    }

    pub fn add_enum_definitions(registry: &mut DefinitionRegistry, enums: Vec<EnumDefinition>) {
        use crate::intermediate::{IntEnum, StringEnum};

        for enum_ in enums {
            match enum_ {
                EnumDefinition::StringEnum(enum_def) => {
                    let def = Definition::StringEnum(StringEnum::from(enum_def));
                    registry.insert(def);
                }

                EnumDefinition::IntEnum(enum_def) => {
                    let def = Definition::IntEnum(IntEnum::from(enum_def));
                    registry.insert(def);
                }
            };
        }
    }

    pub fn add_json_definitions(
        registry: &mut DefinitionRegistry,
        structs: Vec<SchemaJsonDefinition>,
    ) {
        for struct_ in structs {
            let mut struct_def = Json::new(struct_.name, struct_.hash);

            for field in struct_.fields {
                struct_def.add_field(JsonField {
                    index: field.index,
                    name: field.name.clone().into(),
                    key: field.key.clone().into(),
                    type_: convert_json_datatype(&field, registry),
                    optional: field.optional,
                });
            }

            registry.insert(Definition::Json(struct_def));
        }
    }
}

pub fn document_to_definitions(document: Document) -> DefinitionRegistry {
    let mut registry = DefinitionRegistry::new();

    document_to_intermediate::add_enum_definitions(&mut registry, document.0.enum_definitions);
    document_to_intermediate::add_json_definitions(&mut registry, document.0.json_definitions);

    registry
}
