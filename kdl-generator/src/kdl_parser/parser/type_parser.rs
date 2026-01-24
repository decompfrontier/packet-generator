use std::sync::Arc;

use miette::{Severity, SourceSpan};
use winnow::Parser;

use crate::kdl_parser::SourceInfo;
use crate::kdl_parser::parser::type_parser::combinator_solution::Error;
use crate::kdl_parser::{Diagnostic, ParsingError, schema::TypeEncoding};

use crate::kdl_parser::schema::DataType;

pub fn generic_parse(
    input: &str,
    _encoding: Option<TypeEncoding>,
    source_code: &Arc<SourceInfo>,
    span: SourceSpan,
) -> Result<DataType, ParsingError> {
    let input = input.trim();

    let parsing_res = combinator_solution::parse_datatype.parse(input);

    match parsing_res {
        Ok(datatype) => Ok(datatype),

        Err(e) => {
            let inner = e.inner();

            let new_span = {
                // HACK(anri):
                // We should probably _know_ when is the byte starting point of
                // the string, but `kdl-rs` does not appear to expose such span.
                let type_decl_start_len = "type=\"".len();
                // let type_decl_end_len = "\"".len();

                let chars = e.char_span();
                let base = span.offset() + type_decl_start_len;
                let start = base + chars.start;
                let end = base + chars.end;

                SourceSpan::from(start..end)
            };

            Err(ParsingError::from(convert_error_to_diagnostic(
                inner,
                source_code.clone(),
                new_span,
            )))
        }
    }
}

fn convert_error_to_diagnostic(
    error: &Error,
    source_code: Arc<SourceInfo>,
    span: SourceSpan,
) -> Diagnostic {
    let diagnostics: Vec<_> = error
        .context
        .iter()
        .map(|e| Diagnostic {
            message: e.message.clone(),
            severity: e.severity,
            source_info: source_code.clone(),
            span,
            help: e.help.clone(),
            label: None,
            related: vec![],
        })
        .collect();

    match &error.cause {
        Some(diag) => Diagnostic {
            message: diag.message.clone(),
            source_info: source_code,
            span,
            severity: diag.severity,
            label: None,
            help: diag.help.clone(),
            related: diagnostics,
        },

        None => Diagnostic {
            message: "unknown error when parsing type".to_owned(),
            source_info: source_code,
            span,
            severity: Severity::Error,
            label: None,
            help: None,
            related: diagnostics,
        },
    }
}

#[allow(dead_code)]
mod combinator_solution {
    use std::fmt::Display;
    use std::sync::Arc;

    use crate::kdl_parser::schema::{ArraySeparator, BoolEncoding, DataType, TypeEncoding};
    use miette::SourceSpan;
    use winnow::ascii::{alpha1, alphanumeric1, space0, space1};
    use winnow::combinator::{
        alt, cut_err, delimited, not, opt, peek, preceded, separated, separated_pair,
    };
    use winnow::error::{AddContext, FromExternalError, ParserError};
    use winnow::stream::Stream;
    use winnow::token::literal;

    use winnow::prelude::*;

    #[derive(Debug, Clone)]
    pub struct Error {
        pub cause: Option<MiniDiagnostic>,
        pub context: Vec<MiniDiagnostic>,
        pub span: Option<SourceSpan>,
    }

    impl Display for Error {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match &self.cause {
                Some(diag) => {
                    write!(f, "{}", diag)
                }

                None => {
                    write!(f, "unknown error")
                }
            }
        }
    }

    impl std::error::Error for Error {}

    impl miette::Diagnostic for Error {
        fn severity(&self) -> Option<miette::Severity> {
            match &self.cause {
                Some(diag) => diag.severity(),
                None => None,
            }
        }

        fn help<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
            match &self.cause {
                Some(diag) => diag.help(),
                None => None,
            }
        }

        fn related<'a>(
            &'a self,
        ) -> Option<Box<dyn Iterator<Item = &'a dyn miette::Diagnostic> + 'a>> {
            if self.context.is_empty() {
                None
            } else {
                Some(Box::new(
                    self.context.iter().map(|d| d as &dyn miette::Diagnostic),
                ))
            }
        }

        fn diagnostic_source(&self) -> Option<&dyn miette::Diagnostic> {
            None
        }
    }

    #[derive(Debug, Clone, thiserror::Error)]
    #[error("{message}")]
    pub struct MiniDiagnostic {
        pub message: String,
        pub severity: miette::Severity,
        pub help: Option<String>,
    }

    impl miette::Diagnostic for MiniDiagnostic {
        fn severity(&self) -> Option<miette::Severity> {
            Some(self.severity)
        }

        fn help<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
            match &self.help {
                Some(s) => Some(Box::new(s)),
                None => None,
            }
        }
    }

    impl<I> FromExternalError<I, MiniDiagnostic> for Error {
        fn from_external_error(_input: &I, e: MiniDiagnostic) -> Self {
            Self {
                cause: Some(e),
                context: vec![],
                span: None,
            }
        }
    }

    impl<I> FromExternalError<I, Self> for Error {
        fn from_external_error(_input: &I, e: Self) -> Self {
            e
        }
    }

    impl<I: Stream + Clone> ParserError<I> for Error {
        type Inner = Self;

        fn from_input(_input: &I) -> Self {
            Self {
                cause: None,
                context: vec![],
                span: None,
            }
        }

        fn into_inner(self) -> winnow::Result<Self::Inner, Self> {
            Ok(self)
        }
    }

    impl<I: Stream> AddContext<I, MiniDiagnostic> for Error {
        fn add_context(
            mut self,
            _input: &I,
            _token_start: &<I as Stream>::Checkpoint,
            context: MiniDiagnostic,
        ) -> Self {
            match self.cause {
                None => {
                    self.cause = Some(context);

                    self
                }

                Some(old_cause) => {
                    self.context.push(old_cause);

                    self.cause = Some(context);

                    self
                }
            }
        }
    }

    type PResult<T = DataType> = winnow::ModalResult<T, Error>;

    #[derive(Debug)]
    struct Modifier<'a> {
        name: &'a str,
        params: Vec<&'a str>,
    }

    macro_rules! generate_intlike {
        ($fn_name:ident, $type_name:ident, $enum_name:ident) => {
            fn $fn_name(input: &mut &str) -> PResult {
                preceded(
                    (stringify!($type_name), not(peek(alpha1))),
                    cut_err(opt(parse_modifier).try_map(|modifiers| match modifiers {
                        None => Err(MiniDiagnostic {
                            message: format!(
                                "integer-like type `{}` requires an encoding",
                                stringify!($type_name)
                            ),
                            severity: miette::Severity::Error,
                            help: Some(format!(
                                "specify either {0}::int or {0}::str",
                                stringify!($type_name)
                            )),
                        }),

                        Some(modifiers) => {
                            let valid_modifiers =
                                modifiers.iter().filter(|modifier| match modifier.name {
                                    "str" | "int" => true,
                                    _ => false,
                                });

                            let last_encoding = valid_modifiers.last().and_then(|s| match s.name {
                                "int" => Some(TypeEncoding::Int),
                                "str" => Some(TypeEncoding::String),
                                _ => None,
                            });

                            match last_encoding {
                                Some(encoding) => Ok(DataType::$enum_name { encoding }),
                                None => Err(MiniDiagnostic {
                                    message: format!(
                                        "no encoding specified when parsing modifiers of type `{}`",
                                        stringify!($type_name)
                                    ),
                                    severity: miette::Severity::Error,
                                    help: Some(format!(
                                        "specify one of `{0}::str` or `{0}::int`",
                                        stringify!($type_name)
                                    )),
                                }),
                            }
                        }
                    })),
                )
                .parse_next(input)
            }
        };
    }

    pub fn parse_datatype(input: &mut &str) -> PResult {
        alt((
            parse_map,
            parse_array,
            parse_i32,
            parse_i64,
            parse_u32,
            parse_u64,
            parse_f32,
            parse_f64,
            parse_bool,
            parse_str,
            parse_datetime_unix,
            parse_datetime,
            parse_custom_type,
        ))
        .parse_next(input)
    }

    generate_intlike!(parse_i32, i32, I32);
    generate_intlike!(parse_u32, u32, U32);
    generate_intlike!(parse_i64, i64, I64);
    generate_intlike!(parse_u64, u64, U64);
    generate_intlike!(parse_f32, f32, F32);

    fn parse_bool(input: &mut &str) -> PResult {
        preceded(
            ("bool", not(peek(alpha1))),
            cut_err(opt(parse_modifier).map(|modifiers| match modifiers {
                None => DataType::Bool {
                    encoding: BoolEncoding::Bool,
                },

                Some(modifiers) => {
                    let valid_modifiers = modifiers
                        .iter()
                        .rfind(|modifier| matches!(modifier.name, "str" | "int"));

                    let last_encoding = valid_modifiers
                        .and_then(|s| match s.name {
                            "int" => Some(BoolEncoding::Int),
                            "str" => Some(BoolEncoding::String),
                            _ => None,
                        })
                        .unwrap_or(BoolEncoding::Bool);

                    DataType::Bool {
                        encoding: last_encoding,
                    }
                }
            })),
        )
        .parse_next(input)
    }

    fn parse_f64(input: &mut &str) -> PResult {
        preceded("f64", (not(alphanumeric1), opt(parse_modifier)))
            .map(|((), _maybe_modifiers)| DataType::F64)
            .parse_next(input)
    }

    fn parse_str(input: &mut &str) -> PResult {
        preceded("str", (not(alphanumeric1), opt(parse_modifier)))
            .map(|((), _maybe_modifiers)| DataType::String)
            .parse_next(input)
    }

    fn parse_datetime(input: &mut &str) -> PResult {
        preceded("datetime", (not(alphanumeric1), opt(parse_modifier)))
            .map(|((), _maybe_modifiers)| DataType::Datetime)
            .parse_next(input)
    }

    fn parse_datetime_unix(input: &mut &str) -> PResult {
        preceded("datetime-unix", (not(alphanumeric1), opt(parse_modifier)))
            .map(|((), _maybe_modifiers)| DataType::DatetimeUnix)
            .parse_next(input)
    }

    fn parse_custom_type(input: &mut &str) -> PResult {
        (alphanumeric1, (not(alphanumeric1), opt(parse_modifier)))
            .map(|(name, ((), _maybe_modifiers))| DataType::Custom(name.to_owned()))
            .parse_next(input)
    }

    fn parse_map(input: &mut &str) -> PResult {
        delimited(
            ("%{", space0),
            (
                separated_pair(
                    parse_datatype,
                    cut_err((space1, literal("=>"), space1).context(MiniDiagnostic {
                        message: "expected separator ` => ` in map".to_owned(),
                        severity: miette::Severity::Error,
                        help: Some("a map is defined as `%{foo => bar}`.".to_owned()),
                    })),
                    parse_datatype,
                ),
                opt(parse_modifier),
            ),
            (space0, "}"),
        )
        .map(|((x, y), _maybe_modifiers)| DataType::Map {
            key: Arc::new(x),
            value: Arc::new(y),
        })
        .parse_next(input)
    }

    fn parse_array(input: &mut &str) -> PResult {
        #[derive(Debug, Clone)]
        enum PartialArrayType {
            SingleElement,
            StringArray { separator: ArraySeparator },
            NormalArray,
        }

        (delimited("[", parse_datatype, "]"),
            cut_err(opt(parse_modifier).try_map(|maybe_modifiers| {
                match maybe_modifiers {

                None => Ok(PartialArrayType::NormalArray),

                Some(modifiers) => {

                    let possible_array_types: Result<Vec<_>, MiniDiagnostic> = modifiers
                        .iter()
                        .filter_map(|modifier| -> Option<Result<_, MiniDiagnostic>> {
                            match modifier.name {
                                "sep" => {
                                    let x = modifier
                                        .params
                                        .first()
                                        .ok_or_else(|| MiniDiagnostic {
                                            message: "array modifier `sep` needs to include the separator".to_owned(),
                                            severity: miette::Severity::Error,
                                            help: Some("add one of `::sep(at)`, `::sep(colon)`, `::sep(comma)`".to_owned()),
                                        })
                                        .and_then(|&sep| {
                                            match sep {
                                                "at" => Ok(ArraySeparator::At),
                                                "colon" => Ok(ArraySeparator::Colon),
                                                "comma" => Ok(ArraySeparator::Comma),
                                                _ => Err(MiniDiagnostic {
                                                    message: format!("unknown separator {sep}"),
                                                    severity: miette::Severity::Error,
                                                    help: Some("use one of `::sep(at)`, `::sep(colon)`, `::sep(comma)`".to_owned()),
                                                })
                                            }
                                        })
                                        .map(|separator| {
                                            PartialArrayType::StringArray { separator }
                                        });

                                    Some(x)
                                }

                                "size" => {
                                    let x = modifier
                                        .params
                                        .first()
                                        .ok_or_else(|| MiniDiagnostic {
                                            message: "array modifier `size` needs to include the number of elements".to_owned(),
                                            severity: miette::Severity::Error,
                                            help: Some("add one of `::size(1)`, `::size(n)`".to_owned()),
                                        })
                                        .and_then(|&size| {
                                            match size {
                                                "1" => Ok(PartialArrayType::SingleElement),
                                                "n" => Ok(PartialArrayType::NormalArray),
                                                _ => Err(MiniDiagnostic {
                                                    message: format!("unrecognized size `{size}` in array size modifier declaration"),
                                                    severity: (miette::Severity::Error),
                                                    help: Some("use one of `::size(1)` or `::size(n)`".to_owned()),
                                                })
                                            }
                                        });

                                    Some(x)
                                }

                                _ => None,
                            }
                        })
                        .collect();

                    let array_types = possible_array_types?;


                    if array_types.len() > 1 {
                        return Err(MiniDiagnostic {
                            message: "multiple conflicting array modifiers, specify either `::size` or `::sep`, but not both".to_owned(),
                            severity: miette::Severity::Error,
                            help: None,
                        })
                    }

                    match array_types.first() {
                            Some(elem) => Ok(elem.clone()),
                            None => Ok(PartialArrayType::NormalArray),
                    }
                }
            }
            })
            ))
            .map(|(inner, array_type)| {
                match array_type {
                    PartialArrayType::SingleElement => DataType::SingleElementArray(inner.into()),
                    PartialArrayType::NormalArray => DataType::Array(inner.into()),
                    PartialArrayType::StringArray { separator } => DataType::StringArray { inner: inner.into(), separator},
                }
            })
            .parse_next(input)
    }

    fn parse_modifier<'a>(input: &mut &'a str) -> PResult<Vec<Modifier<'a>>> {
        fn modifier_param_1<'a>(input: &mut &'a str) -> PResult<Modifier<'a>> {
            (
                alpha1,
                opt(delimited(
                    '(',
                    separated(1.., alphanumeric1, (',', space1)),
                    ')',
                )),
            )
                .map(|(name, params)| Modifier {
                    name,
                    params: params.unwrap_or_else(Vec::new),
                })
                .parse_next(input)
        }

        preceded(
            "::",
            cut_err(alt((
                delimited(
                    '{',
                    separated(
                        1..,
                        modifier_param_1,
                        (',', cut_err(space1.context(MiniDiagnostic {
                            message: "expected modifiers in modifier list to be separated by comma _and_ a space"
                                .to_owned(),
                            severity: miette::Severity::Error,
                            help: Some("like so: {foo, bar}".to_owned()),
                        }))),
                    ),
                    '}',
                ),
                modifier_param_1.map(|x| vec![x]),
            )))
            .context(MiniDiagnostic {
                message: "expected modifier or modifier list required after '::' in type definition"
                    .to_owned(),
                severity: miette::Severity::Error,
                help: Some(
                    "add either a modifier (`::foo`) or a modifier list (`::{foo, bar}`), note that the latter is separated by comma _and_ space"
                        .to_owned(),
                ),
            }),
        )
        .parse_next(input)
    }

    #[cfg(test)]
    mod tests {
        use crate::kdl_parser::schema::{DataType, TypeEncoding};

        use super::*;

        #[test]
        fn errors_on_unspecified_i32() {
            let mut s = "i32";
            let val = parse_datatype(&mut s);
            println!("{val:?}");
            assert!(val.is_err());
        }

        #[test]
        fn can_parse_string_encoded_i32() {
            let mut s = "i32::str";
            let val = parse_datatype(&mut s);
            assert!(matches!(
                val,
                Ok(DataType::I32 {
                    encoding: TypeEncoding::String
                })
            ));
        }

        #[test]
        fn can_parse_string_encoded_i32_with_extra() {
            let mut s = "i32::{str, foo}";
            let val = parse_datatype(&mut s);
            assert!(matches!(
                val,
                Ok(DataType::I32 {
                    encoding: TypeEncoding::String
                })
            ));
        }

        #[test]
        fn can_parse_int_encoded_i32() {
            let mut s = "i32::int";
            let val = parse_datatype(&mut s);
            assert!(matches!(
                val,
                Ok(DataType::I32 {
                    encoding: TypeEncoding::Int
                })
            ));
        }

        #[test]
        fn arrays_of_primitive_require_encoding() {
            let mut s = "[i32]";
            let val = parse_datatype(&mut s);
            assert!(matches!(val, Err(..)));
        }

        #[test]
        fn can_parse_normal_arrays() {
            let mut s = "[i32::int]";
            let val = parse_datatype(&mut s);
            assert!(matches!(val, Ok(DataType::Array { .. })));
        }

        #[test]
        fn can_parse_string_at_arrays() {
            let mut s = "[str]::sep(at)";
            let val = parse_datatype(&mut s);
            assert!(matches!(
                val,
                Ok(DataType::StringArray {
                    separator: ArraySeparator::At,
                    ..
                })
            ));
        }

        #[test]
        fn can_parse_string_comma_arrays() {
            let mut s = "[str]::sep(comma)";
            let val = parse_datatype(&mut s);
            assert!(matches!(
                val,
                Ok(DataType::StringArray {
                    separator: ArraySeparator::Comma,
                    ..
                })
            ));
        }

        #[test]
        fn can_parse_string_colon_arrays() {
            let mut s = "[str]::sep(colon)";
            let val = parse_datatype(&mut s);
            assert!(matches!(
                val,
                Ok(DataType::StringArray {
                    separator: ArraySeparator::Colon,
                    ..
                })
            ));
        }

        #[test]
        fn can_parse_sized_arrays() {
            let mut s = "[i32::int]::size(1)";
            let val = parse_datatype(&mut s);
            assert!(matches!(val, Ok(DataType::SingleElementArray { .. })));
        }

        #[test]
        fn can_parse_datetime() {
            let mut s = "datetime";
            let val = parse_datatype(&mut s);
            assert!(matches!(val, Ok(DataType::Datetime)));
        }

        #[test]
        fn can_parse_datetime_unix() {
            let mut s = "datetime-unix";
            let val = parse_datatype(&mut s);
            assert!(matches!(val, Ok(DataType::DatetimeUnix)));
        }

        #[test]
        fn can_parse_custom_type() {
            let mut s = "Foo";
            let val = parse_datatype(&mut s);
            assert!(matches!(val, Ok(DataType::Custom(..))));
        }

        #[test]
        fn can_parse_ambigous_custom_type() {
            let vals = &[
                "strange", // starts with "str"
                "i32foo", "f64foo",
            ];

            for &v in vals {
                let mut x = v;
                let val = parse_datatype(&mut x);
                assert!(!matches!(val, Ok(DataType::String)));
                assert!(matches!(val, Ok(DataType::Custom(..))));
            }
        }

        #[test]
        fn can_parse_maps() {
            let mut s = "%{i32::str => i64::str}";
            let val = parse_datatype(&mut s);
            assert!(matches!(val, Ok(DataType::Map { .. })));
        }

        #[test]
        fn can_parse_nested_maps() {
            let mut s = "%{i32::str => %{i64::str => f32::int}}";
            let val = parse_datatype(&mut s);
            assert!(matches!(val, Ok(DataType::Map { .. })));
        }

        #[test]
        fn way_too_many_parenthesis() {
            let mut s = "[[[%{i32::{str} => %{i64::{str, foo(x, y, z)} => f32::{int, bar(x)}}}]::size(1)]::sep(comma)]";
            let val = parse_datatype(&mut s);
            assert!(matches!(val, Ok(DataType::Array { .. })));
        }
    }
}
