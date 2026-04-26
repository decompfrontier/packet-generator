//! Error types and related utilities used in the parser.
//!
//! The main type is [`Diagnostic`] to issue user-facing errors through
//! the [`miette::Diagnostic`] trait.

use miette::{LabeledSpan, MietteSpanContents, Severity, SourceCode, SourceSpan};
use std::{fmt::Display, path::PathBuf, sync::Arc};

use kdl::KdlError;

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

    #[error("no document found")]
    NoDocumentProvided,

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

/// Type that represent non-breaking parsing warnings to the user of the
/// library.
///
/// These should be suggestions about _possibly wrong_ code, but do not
/// interfere with parsing.
#[derive(Debug, Clone, thiserror::Error)]
#[error("warnings/advices generated during parsing")]
pub struct ParsingWarnings {
    // Expose the inner fields to the `crate::kdl_parser` module and nothing more.
    pub(super) source_info: Arc<SourceInfo>,
    pub(super) diagnostics: Vec<Diagnostic>,
}

impl ParsingWarnings {
    /// Checks if warnings were generated.
    #[must_use]
    pub const fn are_there_any(&self) -> bool {
        !self.diagnostics.is_empty()
    }

    /// Prints warnings to stdout if they were present.
    pub fn print_warnings_if_any(self) {
        if !self.diagnostics.is_empty() {
            let report = miette::Report::from(self);
            println!("{report:?}");
        }
    }

    /// Iterator over the inner [`Diagnostic`]s.
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
