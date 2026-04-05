use std::sync::Arc;

use miette::{Severity, SourceSpan};
use winnow::{LocatingSlice, Parser};

use crate::kdl_parser::SourceInfo;
use crate::kdl_parser::parser::type_parser::combinators::Error;
use crate::kdl_parser::{Diagnostic, ParsingError, schema::IntLikeEncoding};

use crate::kdl_parser::schema::DataType;

pub fn generic_parse(
    input: &str,
    _encoding: Option<IntLikeEncoding>,
    source_code: &Arc<SourceInfo>,
    span: SourceSpan,
) -> Result<DataType, ParsingError> {
    let input = combinators::Input {
        input: LocatingSlice::new(input),
        state: combinators::State {},
    };

    let parsing_res = combinators::parse_datatype.parse(input);

    match parsing_res {
        Ok(datatype) => Ok(datatype),

        Err(e) => {
            let inner = e.inner();

            let new_span = {
                // HACK(anri):
                // We have to append `type="` to get the correct offset :(
                let base = span.offset() + "type=\"".len();
                let start = base + e.char_span().start;
                let end = base + e.char_span().end;

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
mod combinators {
    use std::fmt::Display;
    use std::num::NonZeroUsize;
    use std::sync::Arc;

    use crate::kdl_parser::schema::{
        ArraySeparator, ArraySize, BoolEncoding, DataType, IntLikeEncoding, JsonEncoding,
    };
    use miette::SourceSpan;
    use winnow::ascii::{alpha1, alphanumeric1, space0, space1};
    use winnow::combinator::{
        alt, cut_err, delimited, not, opt, peek, preceded, separated, separated_pair,
    };
    use winnow::error::{AddContext, FromExternalError, ParserError};
    use winnow::stream::Stream;
    use winnow::token::literal;

    use winnow::{LocatingSlice, Stateful, prelude::*};

    #[derive(Clone, Debug)]
    pub struct State {}

    pub type Input<'i> = Stateful<LocatingSlice<&'i str>, State>;

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
    enum Modifier<'a> {
        Unnamed {
            name: &'a str,
            params: Vec<&'a str>,
        },

        Named {
            name: &'a str,
            params: Vec<(&'a str, &'a str)>,
        },
    }

    macro_rules! generate_intlike {
        ($fn_name:ident, $type_name:ident, $enum_name:ident) => {
            fn $fn_name(input: &mut Input) -> PResult {
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
                            let last_encoding = modifiers
                                .iter()
                                .filter_map(|modifier| match modifier {
                                    Modifier::Unnamed { name: "str", .. } => {
                                        Some(IntLikeEncoding::String)
                                    }
                                    Modifier::Unnamed { name: "int", .. } => {
                                        Some(IntLikeEncoding::Int)
                                    }
                                    _ => None,
                                })
                                .next_back();

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

    pub fn parse_datatype(input: &mut Input) -> PResult {
        (
            space0,
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
            )),
        )
            .map(|(_space, v)| v)
            .parse_next(input)
    }

    generate_intlike!(parse_i32, i32, I32);
    generate_intlike!(parse_u32, u32, U32);
    generate_intlike!(parse_i64, i64, I64);
    generate_intlike!(parse_u64, u64, U64);
    generate_intlike!(parse_f32, f32, F32);

    fn parse_bool(input: &mut Input) -> PResult {
        preceded(
            ("bool", not(peek(alpha1))),
            cut_err(opt(parse_modifier).map(|modifiers| match modifiers {
                None => DataType::Bool {
                    encoding: BoolEncoding::Bool,
                },

                Some(modifiers) => {
                    let last_specified_encoding = modifiers
                        .iter()
                        .filter_map(|modifier| match modifier {
                            Modifier::Unnamed { name: "str", .. } => Some(BoolEncoding::String),
                            Modifier::Unnamed { name: "int", .. } => Some(BoolEncoding::Int),
                            _ => None,
                        })
                        .next_back();

                    let encoding = last_specified_encoding.unwrap_or(BoolEncoding::Bool);

                    DataType::Bool { encoding }
                }
            })),
        )
        .parse_next(input)
    }

    fn parse_f64(input: &mut Input) -> PResult {
        preceded("f64", (not(alphanumeric1), opt(parse_modifier)))
            .map(|((), _maybe_modifiers)| DataType::F64)
            .parse_next(input)
    }

    fn parse_str(input: &mut Input) -> PResult {
        preceded("str", (not(alphanumeric1), opt(parse_modifier)))
            .map(|((), _maybe_modifiers)| DataType::String)
            .parse_next(input)
    }

    fn parse_datetime(input: &mut Input) -> PResult {
        preceded("datetime", (not(alphanumeric1), opt(parse_modifier)))
            .map(|((), _maybe_modifiers)| DataType::Datetime)
            .parse_next(input)
    }

    fn parse_datetime_unix(input: &mut Input) -> PResult {
        preceded("datetime-unix", (not(alphanumeric1), opt(parse_modifier)))
            .map(|((), _maybe_modifiers)| DataType::DatetimeUnix)
            .parse_next(input)
    }

    fn parse_custom_type(input: &mut Input) -> PResult {
        macro_rules! gen_error {
            (match $var:expr, $(($name:ident -> $expected:expr)),+ $(,)*) => {
                match $var {
                    $(
                    stringify!($name) => Err(MiniDiagnostic {
                        message: String::from(concat!("Found legacy Python datatype `", stringify!($name), "`.")),
                        severity: miette::Severity::Error,
                        help: Some(String::from(concat!("Replace it with `", stringify!($expected), "`."))),
                    }),
                    )+

                    _ => Ok(())
                }
            };
        }

        fn error_on_python_datatype(custom_str: &str) -> Result<(), MiniDiagnostic> {
            gen_error!(match custom_str,
                (int -> i32::int),
                (intstr -> i32::str),
                (long -> i64::int),
                (longstr -> i64::str),
                (float -> f32::int),
                (floatstr -> f32::str),
                (double -> f64::int),
                (doublestr -> f64::str),
                (boolint -> bool::int),
                (intbool -> bool::int),
                (boolstr -> bool::str),
                (strbool -> bool::str),
                (commalist -> "[...]::sep(comma)"),
                (atlist -> "[...]::sep(at)"),
                (colonlist -> "[...]::sep(colon)"),
                (datetimeunix -> "datetime-unix"),
            )
        }

        (alphanumeric1, (not(alphanumeric1), opt(parse_modifier)))
            .try_map(
                |(name, ((), maybe_modifiers))| -> Result<DataType, MiniDiagnostic> {
                    error_on_python_datatype(name)?;

                    let encoding = maybe_modifiers
                        .and_then(|modifiers| {
                            modifiers
                                .iter()
                                .filter_map(|modifier| match modifier {
                                    Modifier::Unnamed { name: "str", .. } => {
                                        Some(JsonEncoding::String)
                                    }
                                    Modifier::Unnamed { name: "json", .. } => {
                                        Some(JsonEncoding::Json)
                                    }
                                    _ => None,
                                })
                                .next_back()
                        })
                        .unwrap_or(JsonEncoding::Json);

                    Ok(DataType::Custom {
                        encoding,
                        name: name.into(),
                    })
                },
            )
            .parse_next(input)
    }

    fn parse_map(input: &mut Input) -> PResult {
        (
            delimited(
                ("%{", space0),
                (separated_pair(
                    parse_datatype,
                    cut_err((space1, literal("=>"), space1).context(MiniDiagnostic {
                        message: "expected separator ` => ` in map".to_owned(),
                        severity: miette::Severity::Error,
                        help: Some("a map is defined as `%{foo => bar}`.".to_owned()),
                    })),
                    parse_datatype,
                ),),
                (space0, "}"),
            ),
            cut_err(opt(parse_modifier)),
        )
            .map(|(((x, y),), _mods)| DataType::Map {
                key: Arc::new(x),
                value: Arc::new(y),
            })
            .parse_next(input)
    }

    fn parse_array(input: &mut Input) -> PResult {
        #[derive(Debug, Clone)]
        enum PartialArrayType {
            StringArray {
                separator: ArraySeparator,
                size: ArraySize,
            },
            NormalArray {
                size: ArraySize,
            },
        }

        (delimited("[", parse_datatype, "]"),
            cut_err(opt(parse_modifier).try_map(|maybe_modifiers| {
                match maybe_modifiers {

                None => Ok(PartialArrayType::NormalArray { size: ArraySize::Dynamic }),

                Some(modifiers) => {

                    let array_sizes =
                        modifiers
                        .iter()
                        .filter_map(|modifier| -> Option<Result<_, MiniDiagnostic>> {

                            match modifier {
                                Modifier::Unnamed { name: "size", params } => {
                                    let res =
                                        params
                                        .first()
                                        .ok_or_else(|| MiniDiagnostic {
                                            message: "array modifier `size` needs to include the number of elements".to_owned(),
                                            severity: miette::Severity::Error,
                                            help: Some("add one of `::size(1)`, `::size(n)`".to_owned()),
                                        })
                                        .and_then(|&size| {
                                            if size == "n" {
                                                 Ok(ArraySize::Dynamic)
                                            } else {
                                                // Try to parse as an integer.
                                                let n = size.parse::<usize>().map_err(|_e| {
                                                        MiniDiagnostic {
                                                            message: format!("unexpected argument value `{size}` in array modifier `size`"),
                                                            severity: miette::Severity::Error,
                                                            help: Some("use one of `::size(1)`, `::size(42)` or `::size(n)`".to_owned()),
                                                        }
                                                    })?;

                                                match NonZeroUsize::new(n) {
                                                    Some(v) => Ok(ArraySize::Fixed(v)),

                                                    None => Err(MiniDiagnostic {
                                                        message: format!("fixed array size `{size}` is <= 0"),
                                                        severity: (miette::Severity::Error),
                                                        help: Some("use a strictly positive size such as `::size(42)`".to_owned()),
                                                    })
                                                }
                                            }
                                        });

                                    Some(res)
                                }

                                Modifier::Named { name: "size", .. } => {
                                    Some(Err(MiniDiagnostic { message: "the modifier `size` should be unnamed, not named!".to_owned(),
                                    severity: miette::Severity::Error,
                                    help: Some(("use `::size(42)` instead of `::size(foo: 42)`").to_owned()) }))
                                }

                                _ => None,
                            }
                        })
                        .collect::<Result<Vec<_>, _>>()?;

                    if array_sizes.len() > 1 {
                        return Err(MiniDiagnostic {
                            message: ("multiple sizes specified for array").into(),
                            severity: miette::Severity::Error,
                            help: Some("please specify only one size between `::size(1)`, `::size(42)`, `::size(n)`".into()),
                        });
                    }

                    let array_size = array_sizes.first().copied().unwrap_or_default();

                    let maybe_separators: Result<Vec<_>, _> = modifiers.iter().filter_map(|modifier| -> Option<Result<_, MiniDiagnostic>> {
                            match modifier {
                                Modifier::Unnamed { name: "sep", params } => {
                                    let x =
                                        params
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
                                                "pipe" => Ok(ArraySeparator::Pipe),
                                                _ => Err(MiniDiagnostic {
                                                    message: format!("unknown separator {sep}"),
                                                    severity: miette::Severity::Error,
                                                    help: Some("use one of `::sep(at)`, `::sep(colon)`, `::sep(pipe)`, `::sep(comma)`".to_owned()),
                                                })
                                            }
                                        })
                                        .map(|separator| {
                                            PartialArrayType::StringArray { separator, size: array_size }
                                        });

                                    Some(x)
                                }

                                Modifier::Named { name: "sep", .. } => {
                                    Some(Err(MiniDiagnostic { message: "the modifier `sep` should be unnamed, not named!".to_owned(),
                                    severity: miette::Severity::Error,
                                    help: Some(("use `::sep(comma)` instead of `::sep(foo: comma)`").to_owned()) }))
                                }

                                _ => None
                            }
                        }).collect();

                    let maybe_separators = maybe_separators?;

                    if maybe_separators.len() > 1 {
                        return Err(MiniDiagnostic {
                            message: ("multiple separators specified for string array").into(),
                            severity: miette::Severity::Error,
                            help: Some("please specify only one separator.".into()),
                        });
                    }

                    Ok(maybe_separators.first().cloned().unwrap_or(PartialArrayType::NormalArray { size: array_size }))
                }
            }
            })
            ))
            .map(|(inner, array_type)| {
                match array_type {
                    PartialArrayType::NormalArray { size} => DataType::Array{
                        size,
                        inner: inner.into()
                    },
                    PartialArrayType::StringArray { size,  separator } => DataType::StringArray { inner: inner.into(), separator, size },
                }
            })
            .parse_next(input)
    }

    fn parse_modifier<'a>(input: &mut Input<'a>) -> PResult<Vec<Modifier<'a>>> {
        fn modifier_unnamed_param<'a>(input: &mut Input<'a>) -> PResult<Modifier<'a>> {
            (
                alphanumeric1,
                opt(delimited(
                    '(',
                    separated(1.., alphanumeric1, (',', space1)),
                    ')',
                )),
            )
                .map(|(name, params)| Modifier::Unnamed {
                    name,
                    params: params.unwrap_or_else(Vec::new),
                })
                .parse_next(input)
        }

        fn modifier_named_param<'a>(input: &mut Input<'a>) -> PResult<Modifier<'a>> {
            type ParsedKeyValueModifier<'a> = (&'a str, Option<Vec<(&'a str, (), &'a str)>>);
            (
                alphanumeric1,
                opt(delimited(
                    '(',
                    separated(
                        1..,
                        (alphanumeric1, (':', space1).void(), alphanumeric1),
                        (',', space1),
                    ),
                    ')',
                )),
            )
                .map(|(name, params): ParsedKeyValueModifier| Modifier::Named {
                    name,
                    params: params
                        .map(|v| {
                            v.iter()
                                .map(|(key, (), value)| (*key, *value))
                                .collect::<Vec<_>>()
                        })
                        .unwrap_or_default(),
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
                        alt((modifier_unnamed_param, modifier_named_param)),
                        (',', cut_err(space1.context(MiniDiagnostic {
                            message: "expected modifiers in modifier list to be separated by comma _and_ a space"
                                .to_owned(),
                            severity: miette::Severity::Error,
                            help: Some("like so: {foo, bar}".to_owned()),
                        }))),
                    ),
                    '}',
                ),
                alt((modifier_unnamed_param, modifier_named_param)).map(|x| vec![x]),
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
        use crate::kdl_parser::schema::{DataType, IntLikeEncoding};

        use super::*;

        use proptest::prelude::*;
        use proptest::strategy::Strategy;

        fn arbitrary_modifier() -> impl Strategy<Value = String> {
            macro_rules! mod_params {
                () => {
                    proptest::collection::vec("[a-zA-Z][a-zA-Z0-9]*", 0..5)
                };
            }

            ("[a-zA-Z][a-zA-Z0-9]*", mod_params!()).prop_map(|(name, params)| {
                if params.is_empty() {
                    name
                } else {
                    format!("{name}({})", params.join(", "))
                }
            })
        }

        fn arbitrary_datatype() -> impl Strategy<Value = String> {
            let leaf = prop_oneof![arbitrary_primitive(), arbitrary_non_primitive()];

            leaf.prop_recursive(20, 200, 3, |inner| {
                prop_oneof![
                    inner.clone().prop_flat_map(|val| { arbitrary_array(val) }),
                    (inner.clone(), inner)
                        .prop_flat_map(|(left, right)| { arbitrary_map(left, right) })
                ]
            })
        }

        fn arbitrary_primitive() -> impl Strategy<Value = String> {
            macro_rules! extra_mods {
                () => {
                    prop::collection::vec(arbitrary_modifier(), 0..3)
                };
            }

            let intlikes = {
                let intlikes = prop_oneof![
                    Just("i32"),
                    Just("u32"),
                    Just("i64"),
                    Just("u64"),
                    Just("f32"),
                ];

                let intlike_encoding = prop_oneof![Just("int"), Just("str")];

                (intlikes, intlike_encoding, extra_mods!()).prop_map(|(outer, encoding, mods)| {
                    if mods.is_empty() {
                        format!("{outer}::{encoding}")
                    } else {
                        format!("{outer}::{{{encoding}, {}}}", mods.join(", "))
                    }
                })
            };

            let bools = {
                let bool_encoding = prop_oneof![
                    Just(None),
                    Just(Some(String::from("int"))),
                    Just(Some(String::from("str"))),
                ];

                (bool_encoding, extra_mods!()).prop_map(|(enc, mods)| {
                    if let Some(enc) = enc {
                        if mods.is_empty() {
                            format!("bool::{enc}")
                        } else {
                            format!("bool::{{{enc}, {}}}", mods.join(", "))
                        }
                    } else {
                        if mods.is_empty() {
                            String::from("bool")
                        } else {
                            format!("bool::{{{}}}", mods.join(", "))
                        }
                    }
                })
            };

            let rest = prop_oneof![
                Just(String::from("str")),
                Just(String::from("f64")),
                Just(String::from("datetime")),
                Just(String::from("datetime-unix")),
            ];

            prop_oneof![
                intlikes,
                bools,
                (rest, extra_mods!()).prop_map(|(name, mods)| {
                    if mods.is_empty() {
                        name
                    } else {
                        format!("{name}::{{{}}}", mods.join(", "))
                    }
                })
            ]
        }

        fn arbitrary_array(inner: String) -> impl Strategy<Value = String> {
            macro_rules! extra_mods {
                () => {
                    prop::collection::vec(arbitrary_modifier(), 0..3)
                };
            }

            let size = prop_oneof![
                Just(String::from("size(n)")),
                (1..=usize::MAX).prop_map(|v| format!("size({v})"))
            ];

            let separator = prop_oneof![
                Just(String::from("sep(comma)")),
                Just(String::from("sep(pipe)")),
                Just(String::from("sep(colon)")),
                Just(String::from("sep(at)")),
            ];

            let size_or_separator = prop_oneof![
                Just(String::new()),
                size.clone().prop_map(|size| format!("::{size}")),
                (size.clone(), extra_mods!()).prop_map(|(size, mods)| {
                    if mods.is_empty() {
                        format!("::{{{size}}}")
                    } else {
                        format!("::{{{size}, {}}}", mods.join(", "))
                    }
                }),
                separator
                    .clone()
                    .prop_map(|separator| format!("::{separator}")),
                (separator.clone(), extra_mods!()).prop_map(|(separator, mods)| {
                    if mods.is_empty() {
                        format!("::{{{separator}}}")
                    } else {
                        format!("::{{{separator}, {}}}", mods.join(", "))
                    }
                }),
                (size, separator, extra_mods!()).prop_map(|(size, separator, mods)| {
                    if mods.is_empty() {
                        format!("::{{{separator}, {size}}}")
                    } else {
                        format!("::{{{separator}, {size}, {}}}", mods.join(", "))
                    }
                })
            ];

            size_or_separator.prop_map(move |extra| format!("[{inner}]{extra}"))
        }

        #[allow(
            clippy::needless_pass_by_value,
            reason = "proptest and rustc throw a fit if we use references"
        )]
        fn arbitrary_map(left: String, right: String) -> impl Strategy<Value = String> {
            (
                Just(format!("%{{ {left} => {right} }}")),
                prop::collection::vec(arbitrary_modifier(), 0..4),
            )
                .prop_map(|(map, mods)| {
                    if mods.is_empty() {
                        map
                    } else {
                        format!("{map}::{{{}}}", mods.join(", "))
                    }
                })
        }

        prop_compose! {
            fn arbitrary_non_primitive()(
                name in "[A-Z][a-zA-Z0-9]*"
            ) -> String {
                name
            }
        }

        proptest! {
            #[test]
            fn test_arbitrary_types(s in arbitrary_datatype()) {
                println!("{s}");
                let mut input = Input {
                    input: LocatingSlice::new(&s),
                    state: State {},
                };
                let val = parse_datatype(&mut input);
                println!("{val:#?}");
                assert!(val.is_ok(), "Error: {}", val.unwrap_err().into_inner().unwrap());

            }
        }

        #[test]
        fn errors_on_unspecified_i32() {
            let s = "i32";
            let mut input = Input {
                input: LocatingSlice::new(s),
                state: State {},
            };
            let val = parse_datatype(&mut input);
            println!("{val:?}");
            assert!(val.is_err());
        }

        #[test]
        fn can_parse_string_encoded_i32() {
            let s = "i32::str";
            let mut input = Input {
                input: LocatingSlice::new(s),
                state: State {},
            };
            let val = parse_datatype(&mut input);
            assert!(matches!(
                val,
                Ok(DataType::I32 {
                    encoding: IntLikeEncoding::String
                })
            ));
        }

        #[test]
        fn can_parse_string_encoded_i32_with_extra() {
            let s = "i32::{str, foo}";
            let mut input = Input {
                input: LocatingSlice::new(s),
                state: State {},
            };
            let val = parse_datatype(&mut input);
            assert!(matches!(
                val,
                Ok(DataType::I32 {
                    encoding: IntLikeEncoding::String
                })
            ));
        }

        #[test]
        fn can_parse_int_encoded_i32() {
            let s = "i32::int";
            let mut input = Input {
                input: LocatingSlice::new(s),
                state: State {},
            };
            let val = parse_datatype(&mut input);
            assert!(matches!(
                val,
                Ok(DataType::I32 {
                    encoding: IntLikeEncoding::Int
                })
            ));
        }

        #[test]
        fn arrays_of_primitive_require_encoding() {
            let s = "[i32]";
            let mut input = Input {
                input: LocatingSlice::new(s),
                state: State {},
            };
            let val = parse_datatype(&mut input);
            assert!(matches!(val, Err(..)));
        }

        #[test]
        fn can_parse_normal_arrays() {
            let s = "[i32::int]";
            let mut input = Input {
                input: LocatingSlice::new(s),
                state: State {},
            };
            let val = parse_datatype(&mut input);
            assert!(matches!(val, Ok(DataType::Array { .. })));
        }

        #[test]
        fn can_parse_string_at_arrays() {
            let s = "[str]::sep(at)";
            let mut input = Input {
                input: LocatingSlice::new(s),
                state: State {},
            };
            let val = parse_datatype(&mut input);
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
            let s = "[str]::sep(comma)";
            let mut input = Input {
                input: LocatingSlice::new(s),
                state: State {},
            };
            let val = parse_datatype(&mut input);
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
            let s = "[str]::sep(colon)";
            let mut input = Input {
                input: LocatingSlice::new(s),
                state: State {},
            };
            let val = parse_datatype(&mut input);
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
            let s = "[i32::int]::size(1)";
            let mut input = Input {
                input: LocatingSlice::new(s),
                state: State {},
            };
            let val = parse_datatype(&mut input);
            assert!(matches!(val, Ok(DataType::Array { .. })));
        }

        #[test]
        fn can_parse_datetime() {
            let s = "datetime";
            let mut input = Input {
                input: LocatingSlice::new(s),
                state: State {},
            };
            let val = parse_datatype(&mut input);
            assert!(matches!(val, Ok(DataType::Datetime)));
        }

        #[test]
        fn can_parse_datetime_unix() {
            let s = "datetime-unix";
            let mut input = Input {
                input: LocatingSlice::new(s),
                state: State {},
            };
            let val = parse_datatype(&mut input);
            assert!(matches!(val, Ok(DataType::DatetimeUnix)));
        }

        #[test]
        fn can_parse_custom_type() {
            let s = "Foo";
            let mut input = Input {
                input: LocatingSlice::new(s),
                state: State {},
            };
            let val = parse_datatype(&mut input);
            assert!(matches!(
                val,
                Ok(DataType::Custom {
                    encoding: JsonEncoding::Json,
                    ..
                })
            ));
        }

        #[test]
        fn can_parse_ambiguous_custom_type() {
            let vals = &[
                "strange", // starts with "str"
                "i32foo", "f64foo",
            ];

            for &v in vals {
                let mut input = Input {
                    input: LocatingSlice::new(v),
                    state: State {},
                };
                let val = parse_datatype(&mut input);
                assert!(!matches!(val, Ok(DataType::String)));
                assert!(matches!(
                    val,
                    Ok(DataType::Custom {
                        encoding: JsonEncoding::Json,
                        ..
                    })
                ));
            }
        }

        #[test]
        fn can_parse_maps() {
            let s = "%{i32::str => i64::str}";
            let mut input = Input {
                input: LocatingSlice::new(s),
                state: State {},
            };
            let val = parse_datatype(&mut input);
            assert!(matches!(val, Ok(DataType::Map { .. })));
        }

        #[test]
        fn can_parse_nested_maps() {
            let s = "%{i32::str => %{i64::str => f32::int}}";
            let mut input = Input {
                input: LocatingSlice::new(s),
                state: State {},
            };
            let val = parse_datatype(&mut input);
            assert!(matches!(val, Ok(DataType::Map { .. })));
        }

        #[test]
        fn way_too_many_parenthesis() {
            let s = "[[[%{i32::{str} => %{i64::{str, foo(x, y, z)} => f32::{int, bar(x)}}}]::size(1)]::sep(comma)]";
            let mut input = Input {
                input: LocatingSlice::new(s),
                state: State {},
            };
            let val = parse_datatype(&mut input);
            assert!(matches!(val, Ok(DataType::Array { .. })));
        }
    }
}
