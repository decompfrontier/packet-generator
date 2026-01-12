use std::sync::Arc;

use miette::{Severity, SourceSpan};

use crate::kdl_parser::{Diagnostic, ParsingError, schema::TypeEncoding};

use crate::kdl_parser::schema::{ArraySeparator, DataType};

fn make_missing_encoding_err(source_code: Arc<str>, span: SourceSpan) -> ParsingError {
    Diagnostic {
        message: "missing `encoding` property in field definition".to_owned(),
        severity: Severity::Error,
        source_code: source_code.clone(),
        span,
        help: Some(
            "Since the field definition has a numeric-like type, you must choose between `encoding=int` or `encoding=str`.\nOtherwise, consider adding the property `default-encoding=str|int` to the data definition itself.".to_owned()
        ),
        label: None,
        related: vec![],
    }.into()
}

pub fn generic_parse(
    input: &str,
    encoding: Option<TypeEncoding>,
    source_code: Arc<str>,
    span: SourceSpan,
) -> Result<DataType, ParsingError> {
    let input = input.trim();

    match (input, encoding) {
        ("i32", Some(encoding)) => Ok(DataType::I32 { encoding }),
        ("u32", Some(encoding)) => Ok(DataType::U32 { encoding }),
        ("i64", Some(encoding)) => Ok(DataType::I64 { encoding }),
        ("u64", Some(encoding)) => Ok(DataType::U64 { encoding }),
        ("f32", Some(encoding)) => Ok(DataType::F32 { encoding }),
        ("f64", _) => Ok(DataType::F64),
        ("bool", Some(encoding)) => Ok(DataType::Bool { encoding }),
        ("i32" | "u32" | "i64" | "u64" | "f32" | "bool", None) => {
            Err(make_missing_encoding_err(source_code.clone(), span))
        }
        ("str", _) => Ok(DataType::String),
        ("datetime", _) => Ok(DataType::Datetime),
        ("datetime-unix", _) => Ok(DataType::DatetimeUnix),

        _ => {
            if let Some(res) = parse_map(input, encoding, source_code.clone(), span) {
                res
            // } else if let Some(res) = parse_tuple(input) {
            //     res
            } else if let Some(res) = parse_array(input, encoding, source_code, span) {
                res
            } else {
                Ok(DataType::Custom(input.to_owned()))
            }
        }
    }
}

fn parse_map(
    input: &str,
    encoding: Option<TypeEncoding>,
    source_code: Arc<str>,
    span: SourceSpan,
) -> Option<Result<DataType, ParsingError>> {
    const ARROW: &str = " => ";

    let input = input.trim();
    if !input.contains(ARROW) {
        return None;
    }

    let parameters: Result<Vec<_>, ParsingError> = input
        .split(ARROW)
        .filter(|c| !c.is_empty())
        .map(|s| generic_parse(s, encoding, source_code.clone(), span))
        .collect();

    let Ok(parameters) = parameters else {
        return Some(Err(
            parameters.expect_err("just checked it contains an error.")
        ));
    };

    if parameters.is_empty() {
        return None;
    }

    let mut diagnostics = vec![];

    if parameters.len() == 1 {
        diagnostics.push(Diagnostic {
            message: "Key or value of map type not provided.".to_owned(),
            severity: Severity::Error,
            source_code: source_code.clone(),
            span,
            help: None,
            label: None,
            related: vec![],
        });
    }

    if parameters.len() == 3 {
        diagnostics.push(
            Diagnostic {
                message:
                    "Too many arrows for map type, use `(tuples)` to allow multiple parameters."
                        .to_owned(),
                severity: Severity::Error,
                source_code: source_code.clone(),
                span,
                help: None,
                label: None,
                related: vec![],
            }
            .into(),
        );
    }

    if !diagnostics.is_empty() {
        return Some(Err(ParsingError::Diagnostics {
            source_code,
            diagnostics,
        }));
    }

    Some(Ok(DataType::Map {
        key: Arc::new(parameters[0].clone()),
        value: Arc::new(parameters[1].clone()),
    }))
}

// #[expect(dead_code, reason = "Disabled parsing logic, may re-enable if needed.")]
// fn parse_tuple(input: &str) -> Option<Result<DataType, String>> {
//     let input = input.trim();
//     if !input.starts_with('(') {
//         return None;
//     }
//
//     if !input.ends_with(')') {
//         return Some(Err(
//             "Tuple type must start with '(' and end with ')'.".to_owned()
//         ));
//     }
//
//     let words: Result<Vec<_>, _> = input[1..(input.len() - 1)]
//         .split(", ")
//         .map(generic_parse)
//         .collect();
//
//     match words {
//         Ok(words) => Some(Ok(DataType::Tuple(words))),
//         Err(e) => Some(Err(e)),
//     }
// }

fn parse_array(
    input: &str,
    #[expect(
        unused,
        reason = "Arrays of numbers are always serialized as strings, so encoding is useless here."
    )]
    encoding: Option<TypeEncoding>,
    source_code: Arc<str>,
    span: SourceSpan,
) -> Option<Result<DataType, ParsingError>> {
    let input = input.trim();
    if !input.starts_with('[') {
        return None;
    }

    if !input.ends_with(']') {
        return Some(Err(Diagnostic {
            message: "List type must start with '[' and end with ']', found `{input}`.".to_owned(),
            severity: Severity::Error,
            source_code: source_code.clone(),
            span,
            help: None,
            label: None,
            related: vec![],
        }
        .into()));
    }

    let words = &input[1..(input.len() - 1)];

    // Try to figure out if we have a separator.
    let (words, separator) = match words.chars().last() {
        Some(',') => (&words[0..(words.len() - 1)], Some(ArraySeparator::Comma)),
        Some('@') => (&words[0..(words.len() - 1)], Some(ArraySeparator::At)),
        Some('|') => (&words[0..(words.len() - 1)], Some(ArraySeparator::Colon)),
        _ => (words, None),
    };

    let val = generic_parse(words, encoding, source_code.clone(), span);

    match (val, separator) {
        (Ok(val), Some(separator)) => Some(Ok(DataType::StringArray {
            inner: Arc::new(val),
            separator,
        })),
        (Ok(val), None) => Some(Ok(DataType::Array(Arc::new(val)))),
        // (Ok(val), None) => Some(Ok(DataType::SingleElementArray(Arc::new(val)))),
        (Err(e), _) => Some(Err(e)),
    }
}
