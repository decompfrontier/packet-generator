use std::{borrow::Cow, sync::Arc};

use kdl::{KdlDocument, KdlNode};
use miette::Severity;

use crate::kdl_parser::{Diagnostic, ParsingError};

pub mod enum_parser;
pub mod json_parser;
pub mod type_parser;

struct ErrorContext<'a> {
    source_code: Arc<str>,
    context: Cow<'a, str>,
    not_found_help: Option<Cow<'a, str>>,
    wrong_type_help: Option<Cow<'a, str>>,
}

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
                source_code: error_context.source_code,
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
                source_code: error_context.source_code.clone(),
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
                source_code: error_context.source_code.clone(),
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
                source_code: error_context.source_code.clone(),
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
                source_code: error_context.source_code.clone(),
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
                    source_code: error_context.source_code.clone(),
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
                source_code: error_context.source_code.clone(),
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
                    source_code: error_context.source_code.clone(),
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
                source_code: error_context.source_code.clone(),
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
                    source_code: error_context.source_code.clone(),
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
                source_code: error_context.source_code.clone(),
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
                source_code: error_context.source_code.clone(),
                span: self.span(),
                help: error_context.not_found_help.map(Cow::into_owned),
                label: None,
                related: vec![],
            })
        })
    }
}
