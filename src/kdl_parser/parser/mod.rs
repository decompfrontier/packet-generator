use std::{
    borrow::Cow,
    collections::HashSet,
    path::{Path, PathBuf},
    sync::Arc,
};

use kdl::{KdlDocument, KdlNode};
use miette::Severity;

use crate::{
    kdl_parser::{
        Diagnostic, ParserOpts, ParsingError, ParsingWarnings, SourceInfo,
        schema::{EnumDefinition, RawDocument},
    },
    vfs::{Vfs, VfsPathBuf},
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
                    "property `{}` for {} does not contain a string",
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
                    "property `{}` for {} does not contain a boolean value (#true or #false)",
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
                    "property `{}` for {} does not contain a string",
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

#[derive(Debug, Clone)]
pub struct UnparsedKdl<'a> {
    content: Cow<'a, str>,
    path: Cow<'a, Path>,
}

impl<'a> UnparsedKdl<'a> {
    #[must_use]
    pub const fn new(content: &'a str, path: &'a Path) -> Self {
        Self {
            content: Cow::Borrowed(content),
            path: Cow::Borrowed(path),
        }
    }

    #[must_use]
    pub const fn new_owned(content: String, path: PathBuf) -> Self {
        Self {
            content: Cow::Owned(content),
            path: Cow::Owned(path),
        }
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

fn extract_unvisited_filepaths<'a, V, N>(
    nodes: N,
    callee_source_info: &'a Arc<SourceInfo>,
    visited_documents: &HashSet<VfsPathBuf>,
    current_directory: impl AsRef<Path>,
    warnings: &mut Vec<Diagnostic>,
) -> Result<Vec<(PathBuf, VfsPathBuf)>, ParsingError>
where
    V: Vfs,
    N: IntoIterator<Item = &'a KdlNode>,
{
    nodes
        .into_iter()
        .filter_map(|node| {
            if node.name().value() != IMPORT_NODE_NAME {
                return None;
            }

            let unexplored_paths = node
                .extract_argument_string(
                    0,
                    ErrorContext {
                        source_info: callee_source_info.clone(),
                        context: "document definition".into(),
                        not_found_help: Some("".into()),
                        wrong_type_help: Some("".into()),
                    },
                )
                .and_then(|path| {
                    let path = current_directory.as_ref().join(PathBuf::from(path));
                    let normalized = V::normalize_path(&path)?;

                    Ok((path, normalized))
                });

            match unexplored_paths {
                Ok((cyclic_path, canonical_path))
                    if visited_documents.contains(&canonical_path) =>
                {
                    let cyclic_path = cyclic_path.to_string_lossy();
                    let source_path = callee_source_info.name.clone();

                    warnings.push(Diagnostic {
                        message: format!("cycle detected when reading \"{cyclic_path}\""),
                        severity: Severity::Warning,
                        source_info: callee_source_info.clone(),
                        span: node.span(),
                        help: Some(format!("{cyclic_path} imports\n-> {source_path}, which imports\n--> {cyclic_path} and so on...\nThe parser will parse \"{cyclic_path}\" once, however keep in mind that this cycle implies that the definitions _may_ have cycles between datatypes.")),
                        label: Some("this file imports the current one".to_owned()),
                        related: vec![
                            Diagnostic {
                                message: "break the cycle between `import`s".to_owned(),
                                severity: Severity::Advice,
                                source_info: callee_source_info.clone(),
                                span: node.span(),
                                help: None,
                                label: None,
                                related: vec![],
                            },
                        ]
                    });

                    None
                }

                _ => Some(unexplored_paths),
            }
        })
        .collect()
}

struct ParseSingleDocumentResult {
    document: RawDocument,
    warnings: ParsingWarnings,
}

fn parse_single_document<V: Vfs>(
    document: &UnparsedKdl<'_>,
    previously_visited_documents: &mut HashSet<VfsPathBuf>,
    opts: &ParserOpts<V>,
) -> Result<ParseSingleDocumentResult, ParsingError> {
    let kdl_document = KdlDocument::parse_v2(&document.content)?;

    let source_info = Arc::new(SourceInfo::new(
        document.path.to_string_lossy(),
        &document.content,
    ));

    let mut erroring_diagnostics = vec![];

    let mut non_erroring_diagnostics = vec![];

    let mut raw_document = RawDocument {
        filepath: Some(document.path.clone().into()),
        json_definitions: vec![],
        http_definitions: vec![],
        enum_definitions: vec![],
    };

    let children = kdl_document.nodes();

    if children.is_empty() {
        non_erroring_diagnostics.push(Diagnostic {
            message: "the file is empty".to_owned(),
            severity: Severity::Warning,
            source_info: source_info.clone(),
            span: kdl_document.span(),
            help: Some("add some definitions".to_owned()),
            label: None,
            related: vec![],
        });

        return Ok(ParseSingleDocumentResult {
            document: raw_document,
            warnings: ParsingWarnings {
                source_info: source_info.clone(),
                diagnostics: non_erroring_diagnostics,
            },
        });
    }

    let current_directory = document
        .path
        .parent()
        .map_or_else(|| PathBuf::from(""), ToOwned::to_owned);

    let unvisited_includes = extract_unvisited_filepaths::<V, _>(
        children,
        &source_info,
        previously_visited_documents,
        &current_directory,
        &mut non_erroring_diagnostics,
    )?;

    for (path, canonical_path) in unvisited_includes {
        let other_document_content = opts.vfs.read_file_to_string(&V::normalize_path(&path)?)?;
        let other_document = UnparsedKdl {
            content: Cow::Owned(other_document_content),
            path: Cow::Owned(path),
        };

        previously_visited_documents.insert(canonical_path);

        let ParseSingleDocumentResult {
            document: other_root,
            warnings: other_warnings,
        } = parse_single_document(&other_document, previously_visited_documents, opts)?;

        raw_document.extend(other_root);
        non_erroring_diagnostics.extend(other_warnings.diagnostics);
    }

    {
        let diagnostics = parse_all_definitions(&mut raw_document, children, &source_info)?;
        erroring_diagnostics.extend(diagnostics);
    }

    if erroring_diagnostics.is_empty() {
        Ok(ParseSingleDocumentResult {
            document: raw_document,
            warnings: ParsingWarnings {
                source_info,
                diagnostics: non_erroring_diagnostics,
            },
        })
    } else {
        Err(ParsingError::Diagnostics {
            source_info,
            diagnostics: erroring_diagnostics,
        })
    }
}

/// Parses one or more `document` (as string) to obtain [`RawDocument`] that can
/// be inspected for further analysis.
///
/// The [`RawDocument`] can be later converted to a
/// [`Document`](super::Document) used for generating the IR by calling
/// the [`RawDocument::finalize`] method.
///
/// # Errors
///
/// Returns `Err` with [`ParsingError`] if there were any errors when parsing
/// the file.
pub fn raw_parse_kdl<V: Vfs>(
    kdl_documents: &[UnparsedKdl<'_>],
    opts: &ParserOpts<V>,
) -> Result<(RawDocument, ParsingWarnings), ParsingError> {
    let mut documents_iter = kdl_documents.iter();

    if let Some(document) = documents_iter.next() {
        let mut visited_paths = HashSet::new();
        visited_paths.insert(V::normalize_path(&document.path)?);

        let ParseSingleDocumentResult {
            mut document,
            mut warnings,
        } = parse_single_document(document, &mut visited_paths, opts)?;

        for other_unparsed_document in documents_iter {
            let vfs_path = V::normalize_path(&other_unparsed_document.path)?;
            if visited_paths.insert(vfs_path) {
                // The file was not visited, so we add it.

                let ParseSingleDocumentResult {
                    document: other_document,
                    warnings: other_warnings,
                } = parse_single_document(other_unparsed_document, &mut visited_paths, opts)?;

                document.extend(other_document);
                warnings.diagnostics.extend(other_warnings.diagnostics);
            }
        }

        Ok((document, warnings))
    } else {
        Err(ParsingError::NoDocumentProvided)
    }
}
