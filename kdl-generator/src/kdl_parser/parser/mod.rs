use std::{
    borrow::Cow,
    collections::HashSet,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
    sync::Arc,
};

use kdl::{KdlDocument, KdlNode};
use miette::Severity;

use crate::kdl_parser::{
    Diagnostic, ParsingError, SourceInfo,
    schema::{EnumDefinition, RawDocument},
};

pub mod enum_parser;
pub mod json_parser;
pub mod type_parser;

/// Used when including other files.
const IMPORT_NODE_NAME: &str = "import";

const JSON_DEFINITION_NAME: &str = "json";
const INT_ENUM_DEFINITION_NAME: &str = "int-enum";
const STRING_ENUM_DEFINITION_NAME: &str = "str-enum";
const XML_DEFINITION_NAME: &str = "xml";
const HTTP_DEFINITION_NAME: &str = "http";
const PLIST_DEFINITION_NAME: &str = "plist";

struct ErrorContext<'a> {
    source_info: Arc<SourceInfo>,
    context: Cow<'a, str>,
    not_found_help: Option<Cow<'a, str>>,
    wrong_type_help: Option<Cow<'a, str>>,
}

#[allow(dead_code, reason = "the remaining methods may be used at some point")]
trait KdlNodeUtilsExt {
    fn extract_argument_string(
        &self,
        index: usize,
        error_context: ErrorContext,
    ) -> Result<&str, ParsingError>;

    fn extract_argument_int(
        &self,
        index: usize,
        error_context: ErrorContext,
    ) -> Result<i128, ParsingError>;

    fn extract_argument_bool(
        &self,
        index: usize,
        error_context: ErrorContext,
    ) -> Result<bool, ParsingError>;

    fn extract_property_string(
        &self,
        property_name: impl AsRef<str>,
        error_context: ErrorContext,
    ) -> Result<&str, ParsingError>;

    fn extract_property_bool(
        &self,
        property_name: impl AsRef<str>,
        error_context: ErrorContext,
    ) -> Result<bool, ParsingError>;

    fn extract_property_int(
        &self,
        property_name: impl AsRef<str>,
        error_context: ErrorContext,
    ) -> Result<i128, ParsingError>;

    fn extract_children(&self, error_context: ErrorContext) -> Result<&KdlDocument, ParsingError>;
}

trait KdlDocumentUtilsExt {
    fn extract_child_node(
        &self,
        name: impl AsRef<str>,
        error_context: ErrorContext,
    ) -> Result<&KdlNode, ParsingError>;
}

impl KdlDocumentUtilsExt for KdlDocument {
    fn extract_child_node(
        &self,
        name: impl AsRef<str>,
        error_context: ErrorContext,
    ) -> Result<&KdlNode, ParsingError> {
        self.get(name.as_ref()).ok_or_else(|| {
            ParsingError::from(Diagnostic {
                message: format!("{} lacks child `{}`", error_context.context, name.as_ref()),
                severity: Severity::Error,
                source_info: error_context.source_info,
                span: self.span(),
                help: error_context.not_found_help.map(Cow::into_owned),
                label: None,
                related: vec![],
            })
        })
    }
}

impl KdlNodeUtilsExt for KdlNode {
    fn extract_argument_string(
        &self,
        index: usize,
        error_context: ErrorContext,
    ) -> Result<&str, ParsingError> {
        let entry = self.entry(index).ok_or_else(|| -> ParsingError {
            ParsingError::from(Diagnostic {
                message: format!(
                    "argument #{} not provided for {}",
                    index + 1,
                    error_context.context
                ),
                severity: Severity::Error,
                source_info: error_context.source_info.clone(),
                span: self.span(),
                help: error_context.not_found_help.map(Cow::into_owned),
                label: None,
                related: vec![],
            })
        })?;

        entry.value().as_string().ok_or_else(|| {
            ParsingError::from(Diagnostic {
                message: format!(
                    "{} (argument #{}) is not a string",
                    error_context.context,
                    index + 1
                ),
                severity: Severity::Error,
                source_info: error_context.source_info.clone(),
                span: entry.span(),
                help: error_context.wrong_type_help.map(Cow::into_owned),
                label: None,
                related: vec![],
            })
        })
    }

    fn extract_argument_bool(
        &self,
        index: usize,
        error_context: ErrorContext,
    ) -> Result<bool, ParsingError> {
        let entry = self.entry(index).ok_or_else(|| -> ParsingError {
            ParsingError::from(Diagnostic {
                message: format!(
                    "argument #{} not provided for {}",
                    index + 1,
                    error_context.context
                ),
                severity: Severity::Error,
                source_info: error_context.source_info.clone(),
                span: self.span(),
                help: error_context.not_found_help.map(Cow::into_owned),
                label: None,
                related: vec![],
            })
        })?;

        entry.value().as_bool().ok_or_else(|| {
            ParsingError::from(Diagnostic {
                message: format!(
                    "{} (argument #{}) is not a boolean",
                    error_context.context,
                    index + 1
                ),
                severity: Severity::Error,
                source_info: error_context.source_info.clone(),
                span: entry.span(),
                help: error_context.wrong_type_help.map(Cow::into_owned),
                label: None,
                related: vec![],
            })
        })
    }

    fn extract_argument_int(
        &self,
        index: usize,
        error_context: ErrorContext,
    ) -> Result<i128, ParsingError> {
        let entry = self.entry(index).ok_or_else(|| -> ParsingError {
            ParsingError::from(Diagnostic {
                message: format!(
                    "argument #{} not provided for {}",
                    index + 1,
                    error_context.context
                ),
                severity: Severity::Error,
                source_info: error_context.source_info.clone(),
                span: self.span(),
                help: error_context.not_found_help.map(Cow::into_owned),
                label: None,
                related: vec![],
            })
        })?;

        entry.value().as_integer().ok_or_else(|| {
            ParsingError::from(Diagnostic {
                message: format!(
                    "{} (argument #{}) is not a string",
                    error_context.context,
                    index + 1
                ),
                severity: Severity::Error,
                source_info: error_context.source_info.clone(),
                span: entry.span(),
                help: error_context.wrong_type_help.map(Cow::into_owned),
                label: None,
                related: vec![],
            })
        })
    }

    fn extract_property_string(
        &self,
        property_name: impl AsRef<str>,
        error_context: ErrorContext,
    ) -> Result<&str, ParsingError> {
        let entry = self
            .entry(property_name.as_ref())
            .ok_or_else(|| -> ParsingError {
                ParsingError::from(Diagnostic {
                    message: format!(
                        "property `{}` not provided for {}",
                        property_name.as_ref(),
                        error_context.context
                    ),
                    severity: Severity::Error,
                    source_info: error_context.source_info.clone(),
                    span: self.span(),
                    help: error_context.not_found_help.map(Cow::into_owned),
                    label: None,
                    related: vec![],
                })
            })?;

        entry.value().as_string().ok_or_else(|| {
            ParsingError::from(Diagnostic {
                message: format!(
                    "property `{}` for {} does not cointain a string",
                    property_name.as_ref(),
                    error_context.context
                ),
                severity: Severity::Error,
                source_info: error_context.source_info.clone(),
                span: entry.span(),
                help: error_context.wrong_type_help.map(Cow::into_owned),
                label: None,
                related: vec![],
            })
        })
    }

    fn extract_property_bool(
        &self,
        property_name: impl AsRef<str>,
        error_context: ErrorContext,
    ) -> Result<bool, ParsingError> {
        let entry = self
            .entry(property_name.as_ref())
            .ok_or_else(|| -> ParsingError {
                ParsingError::from(Diagnostic {
                    message: format!(
                        "property `{}` not provided for {}",
                        property_name.as_ref(),
                        error_context.context
                    ),
                    severity: Severity::Error,
                    source_info: error_context.source_info.clone(),
                    span: self.span(),
                    help: error_context.not_found_help.map(Cow::into_owned),
                    label: None,
                    related: vec![],
                })
            })?;

        entry.value().as_bool().ok_or_else(|| {
            ParsingError::from(Diagnostic {
                message: format!(
                    "property `{}` for {} does not cointain a boolean value (#true or #false)",
                    property_name.as_ref(),
                    error_context.context
                ),
                severity: Severity::Error,
                source_info: error_context.source_info.clone(),
                span: self.span(),
                help: error_context.wrong_type_help.map(Cow::into_owned),
                label: None,
                related: vec![],
            })
        })
    }

    fn extract_property_int(
        &self,
        property_name: impl AsRef<str>,
        error_context: ErrorContext,
    ) -> Result<i128, ParsingError> {
        let entry = self
            .entry(property_name.as_ref())
            .ok_or_else(|| -> ParsingError {
                ParsingError::from(Diagnostic {
                    message: format!(
                        "property `{}` not provided for {}",
                        property_name.as_ref(),
                        error_context.context
                    ),
                    severity: Severity::Error,
                    source_info: error_context.source_info.clone(),
                    span: self.span(),
                    help: error_context.not_found_help.map(Cow::into_owned),
                    label: None,
                    related: vec![],
                })
            })?;

        entry.value().as_integer().ok_or_else(|| {
            ParsingError::from(Diagnostic {
                message: format!(
                    "property `{}` for {} does not cointain a string",
                    property_name.as_ref(),
                    error_context.context
                ),
                severity: Severity::Error,
                source_info: error_context.source_info.clone(),
                span: entry.span(),
                help: error_context.wrong_type_help.map(Cow::into_owned),
                label: None,
                related: vec![],
            })
        })
    }

    fn extract_children(&self, error_context: ErrorContext) -> Result<&KdlDocument, ParsingError> {
        self.children().ok_or_else(|| {
            ParsingError::from(Diagnostic {
                message: format!("{} does not have any child", error_context.context),
                severity: Severity::Error,
                source_info: error_context.source_info.clone(),
                span: self.span(),
                help: error_context.not_found_help.map(Cow::into_owned),
                label: None,
                related: vec![],
            })
        })
    }
}

fn parse_all_definitions(
    raw_document: &mut RawDocument,
    definitions: &[KdlNode],
    source_info: &Arc<SourceInfo>,
) -> Result<Vec<Diagnostic>, ParsingError> {
    let mut all_diagnostics = vec![];

    for (index, definition) in definitions.iter().enumerate() {
        match definition.name().value() {
            JSON_DEFINITION_NAME => {
                match json_parser::parse_data_definition(definition, source_info, index) {
                    Ok(def) => {
                        raw_document.json_definitions.push(def);
                    }

                    Err(ParsingError::Diagnostics { diagnostics, .. }) => {
                        all_diagnostics.extend(diagnostics);
                    }

                    Err(e) => return Err(e),
                }
            }

            INT_ENUM_DEFINITION_NAME => {
                match enum_parser::parse_int_enum_definition(definition, source_info, index) {
                    Ok(def) => {
                        raw_document
                            .enum_definitions
                            .push(EnumDefinition::IntEnum(def));
                    }

                    Err(ParsingError::Diagnostics { diagnostics, .. }) => {
                        all_diagnostics.extend(diagnostics);
                    }

                    Err(e) => return Err(e),
                }
            }

            STRING_ENUM_DEFINITION_NAME => {
                match enum_parser::parse_string_enum_definition(definition, source_info, index) {
                    Ok(def) => {
                        raw_document
                            .enum_definitions
                            .push(EnumDefinition::StringEnum(def));
                    }

                    Err(ParsingError::Diagnostics { diagnostics, .. }) => {
                        all_diagnostics.extend(diagnostics);
                    }

                    Err(e) => return Err(e),
                }
            }

            HTTP_DEFINITION_NAME | XML_DEFINITION_NAME | PLIST_DEFINITION_NAME => {}

            // Ignore the "import" node.
            #[allow(
                clippy::match_same_arms,
                reason = "ignore the `import` node since it is not a definition"
            )]
            IMPORT_NODE_NAME => {}

            other => {
                all_diagnostics.push(Diagnostic {
                    message: format!("unrecognized node `{other}`"),
                    severity: Severity::Warning,
                    source_info: source_info.clone(),
                    span: definition.span(),
                    help: None,
                    label: None,
                    related: vec![],
                });
            }
        }
    }

    Ok(all_diagnostics)
}

fn extract_unvisited_filepath(
    node: &KdlNode,
    callee_source_info: Arc<SourceInfo>,
    visited_documents: &HashSet<PathBuf>,
    current_directory: impl AsRef<Path>,
) -> Option<Result<(PathBuf, PathBuf), ParsingError>> {
    if node.name().value() != IMPORT_NODE_NAME {
        return None;
    }

    let unexplored_paths = node
        .extract_argument_string(
            0,
            ErrorContext {
                source_info: callee_source_info,
                context: "document definition".into(),
                not_found_help: Some("".into()),
                wrong_type_help: Some("".into()),
            },
        )
        .and_then(|path| {
            let path = current_directory.as_ref().join(PathBuf::from(path));

            path.canonicalize()
                .map(|canonical| (path.clone(), canonical))
                .map_err(|e| ParsingError::NoAbsoluteFilePath { path, source: e })
        });

    match unexplored_paths {
        Ok((_cyclic_path, ref canonical_path)) if visited_documents.contains(canonical_path) => {
            // TODO(anri):
            // Emit a warning about there being a cycle between definitions.
            // Requires warnings to not be treated as errors.
            //
            //
            // let path_as_str = cyclic_path.to_string_lossy();
            // Some(Err(ParsingError::from(Diagnostic {
            //     message: format!(
            //         "cyclic reference between {IMPORT_NODE_NAME}s, tried to {IMPORT_NODE_NAME} \"{path_as_str}\""
            //     ),
            //     severity: Severity::Advice,
            //     source_info: callee_source_info,
            //     span: node.span(),
            //     help: None,
            //     label: None,
            //     related: vec![],
            // })))

            None
        }
        _ => Some(unexplored_paths),
    }
}

fn parse_single_document<S: AsRef<str>>(
    document: S,
    filepath: &Path,
    visited_documents: &mut HashSet<PathBuf>,
) -> Result<RawDocument, ParsingError> {
    let kdl_document = KdlDocument::parse_v2(document.as_ref())?;

    let source_info = Arc::new(SourceInfo::new(filepath.to_string_lossy(), document));

    let mut all_diagnostics = vec![];

    let mut raw_document = RawDocument {
        filepath: Some(filepath.into()),
        json_definitions: vec![],
        http_definitions: vec![],
        enum_definitions: vec![],
    };

    let children = kdl_document.nodes();

    if children.is_empty() {
        let diag = Diagnostic {
            message: "the file is empty".to_owned(),
            severity: Severity::Error,
            source_info: source_info.clone(),
            span: kdl_document.span(),
            help: Some("where is everyone?".to_owned()),
            label: None,
            related: vec![],
        };

        all_diagnostics.push(diag);

        return Err(ParsingError::Diagnostics {
            source_info,
            diagnostics: all_diagnostics,
        });
    }

    let current_directory = filepath
        .parent()
        .map_or_else(|| PathBuf::from(""), ToOwned::to_owned);

    let unvisited_includes = children
        .iter()
        .filter_map(|node| {
            extract_unvisited_filepath(
                node,
                source_info.clone(),
                visited_documents,
                &current_directory,
            )
        })
        .collect::<Result<Vec<_>, ParsingError>>()?;

    for (path, canonical_path) in unvisited_includes {
        let mut other_document = String::new();
        let _ = File::open(&path)?.read_to_string(&mut other_document);
        visited_documents.insert(canonical_path);
        let other_root = parse_single_document(other_document, &path, visited_documents)?;
        raw_document
            .json_definitions
            .extend(other_root.json_definitions);
        raw_document
            .http_definitions
            .extend(other_root.http_definitions);
        raw_document
            .enum_definitions
            .extend(other_root.enum_definitions);
    }

    {
        let diagnostics = parse_all_definitions(&mut raw_document, children, &source_info)?;
        all_diagnostics.extend(diagnostics);
    }

    if all_diagnostics.is_empty() {
        Ok(raw_document)
    } else {
        Err(ParsingError::Diagnostics {
            source_info,
            diagnostics: all_diagnostics,
        })
    }
}

/// This function returns a `RawDocument` that can be inspected.
/// A `RawDocument` can be later converted to a `Document` used for generating the IR.
pub fn raw_parse_kdl<S: AsRef<str>>(
    document: S,
    filepath: &Path,
) -> Result<RawDocument, ParsingError> {
    let root_canonical_path =
        filepath
            .canonicalize()
            .map_err(|e| ParsingError::NoAbsoluteFilePath {
                path: filepath.to_path_buf(),
                source: e,
            })?;

    let mut visited_documents = HashSet::from_iter([root_canonical_path]);
    parse_single_document(document, filepath, &mut visited_documents)
}
