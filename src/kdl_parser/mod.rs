//! Module for parsing the definitions (in KDL).
// TODO(anri): rename the module.
//! Continued

mod parser;
pub mod schema;

use std::{fmt::Display, path::PathBuf, sync::Arc};

use miette::{LabeledSpan, MietteSpanContents, Severity, SourceCode, SourceSpan};
pub use schema::validate;

use kdl::KdlError;

/// A valid KDL document.
///
/// This is the only type that can be converted to the IR, later used by
/// the generator, by calling [`document_to_definitions`].
pub struct Document(RawDocument);

use crate::{
    intermediate::{DefinitionRegistry, PartialDefinitionRegistry},
    kdl_parser::schema::RawDocument,
    vfs::Vfs,
};

pub use parser::raw_parse_kdl;

/// Information about the origin of a KDL document.
///
/// Used for generating correct [`Diagnostic`]s.
#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq)]
pub struct SourceInfo {
    /// Simple name (relative path) of the source code.
    pub name: String,

    /// The actual source code.
    pub source_code: String,
}

impl SourceInfo {
    /// Returns a new [`SourceInfo`].
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
                self.name.clone(),
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

/// Main diagnostic type for the parser.
///
/// This type contains every information needed to resolve one
/// Error/Warning/Advice (see [`Severity`]) and related diagnostics.
#[derive(Debug, Clone, thiserror::Error)]
pub struct Diagnostic {
    /// The message to display to the end-user.
    pub message: String,

    /// The [`Severity`] of the diagnostic.
    pub severity: Severity,

    /// The original [information](SourceInfo) about the source code.
    pub source_info: Arc<SourceInfo>,

    /// The byte span where the diagnostic must apply.
    pub span: SourceSpan,

    /// Optional help string to display to the end-user.
    pub help: Option<String>,

    /// Optional label that will be display to the end-user.
    ///
    /// If `None` then by default `"here"` will be used.
    ///
    /// # Example
    ///
    /// Setting `label` to `Some(String::from("foo"))` will display the
    /// following:
    ///
    /// ```text
    /// some error here
    ///            ^^^^
    ///            foo
    /// ```
    ///
    /// Where the `foo` is the `label`.
    pub label: Option<String>,

    /// Related diagnostics.
    pub related: Vec<Self>,
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
        if self.related.is_empty() {
            None
        } else {
            Some(Box::new(
                self.related.iter().map(|d| d as &dyn miette::Diagnostic),
            ))
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
            Self::KdlError(kdl_error) => kdl_error.code(),
            _ => None,
        }
    }

    fn severity(&self) -> Option<miette::Severity> {
        match self {
            Self::KdlError(kdl_error) => kdl_error.severity(),
            _ => None,
        }
    }

    fn help<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        match self {
            Self::KdlError(kdl_error) => kdl_error.help(),
            _ => None,
        }
    }

    fn url<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        match self {
            Self::KdlError(kdl_error) => kdl_error.url(),
            _ => None,
        }
    }

    fn source_code(&self) -> Option<&dyn miette::SourceCode> {
        match self {
            Self::KdlError(kdl_error) => kdl_error.source_code(),
            Self::Diagnostics {
                source_info: source_code,
                ..
            } => Some(source_code),
            _ => None,
        }
    }

    fn labels(&self) -> Option<Box<dyn Iterator<Item = miette::LabeledSpan> + '_>> {
        match self {
            Self::KdlError(kdl_error) => kdl_error.labels(),
            _ => None,
        }
    }

    fn related<'a>(&'a self) -> Option<Box<dyn Iterator<Item = &'a dyn miette::Diagnostic> + 'a>> {
        match self {
            Self::KdlError(kdl_error) => kdl_error.related(),
            Self::Diagnostics { diagnostics, .. } => Some(Box::new(
                diagnostics.iter().map(|d| d as &dyn miette::Diagnostic),
            )),

            _ => None,
        }
    }

    fn diagnostic_source(&self) -> Option<&dyn miette::Diagnostic> {
        match self {
            Self::KdlError(kdl_error) => kdl_error.diagnostic_source(),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, thiserror::Error)]
#[error("warnings/advices generated during parsing")]
pub struct ParsingWarnings {
    source_info: Arc<SourceInfo>,
    diagnostics: Vec<Diagnostic>,
}

impl ParsingWarnings {
    #[must_use]
    pub const fn are_there_any(&self) -> bool {
        !self.diagnostics.is_empty()
    }

    pub fn print_warnings_if_any(self) {
        if !self.diagnostics.is_empty() {
            let report = miette::Report::from(self);
            println!("{report:?}");
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Diagnostic> {
        self.diagnostics.iter()
    }
}

impl miette::Diagnostic for ParsingWarnings {
    fn source_code(&self) -> Option<&dyn SourceCode> {
        Some(&self.source_info)
    }

    fn severity(&self) -> Option<Severity> {
        Some(Severity::Warning)
    }

    fn related<'a>(&'a self) -> Option<Box<dyn Iterator<Item = &'a dyn miette::Diagnostic> + 'a>> {
        Some(Box::new(
            self.diagnostics
                .iter()
                .map(|d| d as &dyn miette::Diagnostic),
        ))
    }
}

/// Common options for the parser.
///
/// The trait parameter `V` specify which VFS to use.
#[derive(Debug, Clone)]
pub struct ParserOpts<V = crate::vfs::DefaultFS>
where
    V: Vfs,
{
    /// The VFS to use.
    vfs: V,
}

impl Default for ParserOpts {
    fn default() -> Self {
        Self {
            vfs: crate::vfs::DefaultFS,
        }
    }
}

impl<V: Vfs> ParserOpts<V> {
    pub const fn new(vfs: V) -> Self {
        Self { vfs }
    }
}

// TODO(anri): move this module somewhere? Possibly try to find a better name.
mod document_to_intermediate {
    use std::sync::Arc;

    use super::schema::EnumDefinition;
    use crate::{
        intermediate::{
            self, BoolEncoding as IntermediateBoolEncoding, DataType as IntermediateDataType,
            Definition, Encoding, Json, JsonField, PartialDefinitionRegistry,
        },
        kdl_parser::schema::{
            self, BoolEncoding, DataType as SchemaDataType, IntLikeEncoding,
            JsonDefinition as SchemaJsonDefinition, JsonField as SchemaJsonField,
        },
    };

    fn convert_datatype_recursive(
        type_: &schema::DataType,
        registry: &mut PartialDefinitionRegistry,
    ) -> intermediate::DataType {
        match type_ {
            schema::DataType::I32 { encoding } => match encoding {
                IntLikeEncoding::String => IntermediateDataType::I32 {
                    encoding: Encoding::String,
                },
                IntLikeEncoding::Int => IntermediateDataType::I32 {
                    encoding: Encoding::Int,
                },
            },

            SchemaDataType::U32 { encoding } => match encoding {
                IntLikeEncoding::String => IntermediateDataType::U32 {
                    encoding: Encoding::String,
                },
                IntLikeEncoding::Int => IntermediateDataType::U32 {
                    encoding: Encoding::Int,
                },
            },

            SchemaDataType::I64 { encoding } => match encoding {
                IntLikeEncoding::String => IntermediateDataType::I64 {
                    encoding: Encoding::String,
                },
                IntLikeEncoding::Int => IntermediateDataType::I64 {
                    encoding: Encoding::Int,
                },
            },

            SchemaDataType::U64 { encoding } => match encoding {
                IntLikeEncoding::String => IntermediateDataType::U64 {
                    encoding: Encoding::String,
                },
                IntLikeEncoding::Int => IntermediateDataType::U64 {
                    encoding: Encoding::Int,
                },
            },

            SchemaDataType::F32 { encoding } => match encoding {
                IntLikeEncoding::String => IntermediateDataType::F32 {
                    encoding: Encoding::String,
                },
                IntLikeEncoding::Int => IntermediateDataType::F32 {
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

            SchemaDataType::Array { inner, size } => IntermediateDataType::Array {
                inner_type: Arc::new(convert_datatype_recursive(inner, registry)),
                size: (*size).into(),
            },

            SchemaDataType::StringArray {
                inner,
                separator,
                size,
            } => {
                use crate::kdl_parser::schema;

                let size = (*size).into();

                match separator {
                    schema::ArraySeparator::Comma => intermediate::DataType::StringArray {
                        separator: intermediate::ArraySeparator::Comma,
                        inner_type: Arc::new(convert_datatype_recursive(inner, registry)),
                        size,
                    },

                    schema::ArraySeparator::Pipe => intermediate::DataType::StringArray {
                        separator: intermediate::ArraySeparator::Pipe,
                        inner_type: Arc::new(convert_datatype_recursive(inner, registry)),
                        size,
                    },

                    schema::ArraySeparator::At => intermediate::DataType::StringArray {
                        separator: intermediate::ArraySeparator::At,
                        inner_type: Arc::new(convert_datatype_recursive(inner, registry)),
                        size,
                    },

                    schema::ArraySeparator::Colon => intermediate::DataType::StringArray {
                        separator: intermediate::ArraySeparator::Colon,
                        inner_type: Arc::new(convert_datatype_recursive(inner, registry)),
                        size,
                    },
                }
            }

            SchemaDataType::Map { key, value } => intermediate::DataType::Map {
                key: Arc::new(convert_datatype_recursive(key, registry)),
                value: Arc::new(convert_datatype_recursive(value, registry)),
            },

            SchemaDataType::Custom { encoding, name } => {
                if let Some(idx) = registry.find_weak(name) {
                    intermediate::DataType::Definition {
                        encoding: (*encoding).into(),
                        definition: idx,
                    }
                } else {
                    intermediate::DataType::Unknown {
                        encoding: (*encoding).into(),
                        name: name.to_owned(),
                    }
                }
            }
        }
    }

    fn convert_json_datatype(
        schema_field: &SchemaJsonField,
        registry: &mut PartialDefinitionRegistry,
    ) -> IntermediateDataType {
        convert_datatype_recursive(&schema_field.r#type, registry)
    }

    pub fn add_enum_definitions(
        registry: &mut PartialDefinitionRegistry,
        enums: Vec<EnumDefinition>,
    ) {
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
            }
        }
    }

    pub fn add_json_definitions(
        registry: &mut PartialDefinitionRegistry,
        structs: Vec<SchemaJsonDefinition>,
    ) {
        for struct_ in structs {
            let mut struct_def = Json::new(
                struct_.name,
                struct_.index,
                struct_.hash,
                struct_.doc,
                struct_.source_info,
                struct_.span,
            );

            for field in struct_.fields {
                struct_def.add_field(JsonField {
                    index: field.index,
                    name: field.name.clone().into(),
                    key: field.key.clone(),
                    type_: convert_json_datatype(&field, registry),
                    optional: field.optional,
                    doc: field.doc,
                    span: field.span,
                });
            }

            registry.insert(Definition::Json(struct_def));
        }
    }
}

#[must_use = "Converting a `Document` to the IR representation implies that you want to use the resulting registry."]
#[allow(clippy::result_large_err, reason = "We can take the performance hit.")]
/// Converts a [`Document`] into a [`DefinitionRegistry`] (IR), returning
/// diangostics in the process.
///
/// This is the only entrypoint for converting the result of KDL's parser into
/// the intermediate representation used by a
/// [`Generator`](crate::generators::Generator).
///
/// # Errors
///
/// Returns [`Diagnostic`] on error if validation fails.
/// In particular, this step checks and resolves every usage of a
/// [`Definition`](crate::intermediate::Definition):
/// a [`Diagnostic`] is returned if an unknown
/// [`Definition`](crate::intermediate::Definition) is used.
pub fn document_to_definitions(document: Document) -> Result<DefinitionRegistry, Diagnostic> {
    let mut registry = PartialDefinitionRegistry::new();

    document_to_intermediate::add_enum_definitions(&mut registry, document.0.enum_definitions);
    document_to_intermediate::add_json_definitions(&mut registry, document.0.json_definitions);

    registry.finalize()
}
