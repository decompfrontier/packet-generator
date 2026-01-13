mod parser;
pub mod schema;

use std::{fmt::Display, str::FromStr, sync::Arc};

use miette::{LabeledSpan, Severity, SourceSpan};
pub use schema::validate;

use kdl::{KdlDocument, KdlError};

pub struct Document(RawDocument);

use crate::{
    intermediate::DefinitionRegistry,
    kdl_parser::{
        parser::enum_parser,
        schema::{EnumDefinition, RawDocument},
    },
};

const JSON_DEFINITION_NAME: &str = "json";
const INT_ENUM_DEFINITION_NAME: &str = "int-enum";
const STRING_ENUM_DEFINITION_NAME: &str = "str-enum";
const XML_DEFINITION_NAME: &str = "xml";
const HTTP_DEFINITION_NAME: &str = "http";
const PLIST_DEFINITION_NAME: &str = "plist";

#[derive(Debug, Clone, thiserror::Error)]
pub struct Diagnostic {
    pub message: String,
    pub severity: Severity,
    pub source_code: Arc<str>,
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
        Some(&self.source_code)
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

    #[error("failed to parse packet definition")]
    Diagnostics {
        source_code: Arc<str>,

        diagnostics: Vec<Diagnostic>,
    },
}

impl From<Diagnostic> for ParsingError {
    fn from(diag: Diagnostic) -> Self {
        Self::Diagnostics {
            source_code: diag.source_code.clone(),
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
            ParsingError::Diagnostics { source_code, .. } => Some(source_code),
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
        }
    }

    fn diagnostic_source(&self) -> Option<&dyn miette::Diagnostic> {
        match self {
            ParsingError::KdlError(kdl_error) => kdl_error.diagnostic_source(),
            _ => None,
        }
    }
}

pub fn raw_parse_kdl<S: AsRef<str>>(document: S) -> Result<schema::RawDocument, ParsingError> {
    let raw_document = KdlDocument::from_str(document.as_ref())?;

    let source_code: Arc<str> = document.as_ref().into();

    let mut all_diagnostics = vec![];

    let mut root = RawDocument {
        json_definitions: vec![],
        http_definitions: vec![],
        enum_definitions: vec![],
    };

    let children = raw_document.nodes();

    if children.is_empty() {
        let diag = Diagnostic {
            message: "the file is empty".to_owned(),
            severity: Severity::Error,
            source_code: source_code.clone(),
            span: raw_document.span(),
            help: Some("where is everyone?".to_owned()),
            label: None,
            related: vec![],
        };

        all_diagnostics.push(diag);

        return Err(ParsingError::Diagnostics {
            source_code: source_code.clone(),
            diagnostics: all_diagnostics,
        });
    }

    // TODO(anri):
    // Parse some meta about versions and stuff?
    //
    // meta {
    //   version 1
    // }
    //
    // let meta = children.iter().filter(|&node| {
    //     node.name().value() == "meta"
    // }).last();
    //
    // match meta {
    //     Some(node) => {
    //
    //     }
    //
    //     None => {
    //
    //     }
    //
    // }

    for definition in children {
        let source_code = source_code.clone();

        match definition.name().value() {
            JSON_DEFINITION_NAME => {
                match parser::json_parser::parse_data_definition(definition, source_code.clone()) {
                    Ok(def) => {
                        root.json_definitions.push(def);
                    }

                    Err(ParsingError::Diagnostics { diagnostics, .. }) => {
                        all_diagnostics.extend(diagnostics);
                    }

                    Err(e) => return Err(e),
                }
            }

            INT_ENUM_DEFINITION_NAME => {
                match enum_parser::parse_int_enum_definition(definition, source_code.clone()) {
                    Ok(def) => {
                        root.enum_definitions.push(EnumDefinition::IntEnum(def));
                    }

                    Err(ParsingError::Diagnostics { diagnostics, .. }) => {
                        all_diagnostics.extend(diagnostics);
                    }

                    Err(e) => return Err(e),
                }
            }

            STRING_ENUM_DEFINITION_NAME => {
                match enum_parser::parse_string_enum_definition(definition, source_code.clone()) {
                    Ok(def) => {
                        root.enum_definitions.push(EnumDefinition::StringEnum(def));
                    }

                    Err(ParsingError::Diagnostics { diagnostics, .. }) => {
                        all_diagnostics.extend(diagnostics);
                    }

                    Err(e) => return Err(e),
                }
            }

            HTTP_DEFINITION_NAME => {}

            XML_DEFINITION_NAME => {}

            PLIST_DEFINITION_NAME => {}

            _ => {}
        }
    }

    if all_diagnostics.is_empty() {
        Ok(root)
    } else {
        Err(ParsingError::Diagnostics {
            source_code,
            diagnostics: all_diagnostics,
        })
    }
}

// TODO(anri): move this module somewhere? Possibly try to find a better name.
mod document_to_intermediate {
    use std::sync::Arc;

    use super::schema::EnumDefinition;
    use crate::{
        intermediate::{
            self, DataType as IntermediateDataType, Definition, DefinitionRegistry, Encoding, Json,
            JsonField,
        },
        kdl_parser::schema::{
            self, DataType as SchemaDataType, JsonDefinition as SchemaJsonDefinition,
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
                TypeEncoding::String => IntermediateDataType::Bool {
                    encoding: Encoding::String,
                },
                TypeEncoding::Int => IntermediateDataType::Bool {
                    encoding: Encoding::Int,
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
                    name: field.name.clone().into(),
                    key: field.key.clone().into(),
                    type_: convert_json_datatype(&field, registry),
                });
            }

            registry.insert(Definition::Struct(struct_def));
        }
    }
}

pub fn document_to_definitions(document: Document) -> DefinitionRegistry {
    let mut registry = DefinitionRegistry::new();

    document_to_intermediate::add_enum_definitions(&mut registry, document.0.enum_definitions);
    // document_to_intermediate::add_json_definitions(&mut registry, document.0.http_definitions);
    document_to_intermediate::add_json_definitions(&mut registry, document.0.json_definitions);

    registry
}
