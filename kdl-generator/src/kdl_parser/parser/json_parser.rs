use std::{str::FromStr, sync::Arc};

use kdl::KdlNode;
use miette::Severity;

use crate::kdl_parser::{
    Diagnostic, ParsingError, SourceInfo,
    parser::{ErrorContext, KdlDocumentUtilsExt, KdlNodeUtilsExt, type_parser::generic_parse},
    schema::{DataType, JSONKey, JsonDefinition, JsonField, TypeEncoding},
};

const DEFAULT_ENCODING_PROPERTY: &str = "default-encoding";
const TYPE_PROPERTY: &str = "type";

const HASH_CHILD: &str = "hash";
const DOC_CHILD: &str = "doc";
const KEY_CHILD: &str = "key";

const FIELD_DEFINITOIN: &str = "field";

const TRANSPARENT_PROPERTY_NAME: &str = "transparent";

pub fn parse_data_definition(
    definition: &KdlNode,
    source_code: Arc<SourceInfo>,
    index: usize,
) -> Result<JsonDefinition, ParsingError> {
    let name = definition.extract_argument_string(
        0,
        ErrorContext {
            source_info: source_code.clone(),
            context: "definition".into(),
            not_found_help: Some("add a name to the definition".into()),
            wrong_type_help: Some("give it a name as a string".into()),
        },
    )?;

    let maybe_default_encoding = definition
        .get(DEFAULT_ENCODING_PROPERTY)
        .map(|prop| {
            prop.as_string().ok_or_else(|| {
                ParsingError::from(Diagnostic {
                    message: format!("property `{DEFAULT_ENCODING_PROPERTY}` in JSON definition `Foo` is not a string"),
                    severity: Severity::Error,
                    source_info: source_code.clone(),
                    span: definition.span(),
                    help: Some("provide one of `str` or `int`".to_owned()),
                    label: None,
                    related: vec![],
                })
            })
        })
        .transpose()?
        .map(TypeEncoding::from_str)
        .transpose()
        .map_err(|e| {
            ParsingError::from(Diagnostic {
                message: e,
                severity: Severity::Warning,
                source_info: source_code.clone(),
                span: definition.span(),
                help: None,
                label: None,
                related: vec![],
            })
        })?;

    let data_children = definition.children().ok_or_else(|| {
        ParsingError::from(Diagnostic {
            message: "JSON definition has no children".to_owned(),
            severity: Severity::Error,
            source_info: source_code.clone(),
            span: definition.span(),
            help: Some(format!(
                "specify children `{HASH_CHILD}`, `{DOC_CHILD}` and some `{FIELD_DEFINITOIN}`s"
            )),
            label: None,
            related: vec![],
        })
    })?;

    let hash = data_children
        .get(HASH_CHILD)
        .map(|node| {
            node.extract_argument_string(
                0,
                ErrorContext {
                    source_info: source_code.clone(),
                    context: "JSON definition `{name}`".into(),
                    not_found_help: None,
                    wrong_type_help: None,
                },
            )
        })
        .transpose()?
        .map(|s| s.to_owned());

    let doc = data_children
        .extract_child_node(
            DOC_CHILD,
            ErrorContext {
                source_info: source_code.clone(),
                context: format!("JSON definition `{name}`").into(),
                not_found_help: Some(format!("specify child `{DOC_CHILD} \"Example\"`").into()),
                wrong_type_help: None,
            },
        )?
        .extract_argument_string(
            0,
            ErrorContext {
                source_info: source_code.clone(),
                context: format!("JSON definition `{name}`").into(),
                not_found_help: Some(format!("specify child `{DOC_CHILD} \"Example\"`").into()),
                wrong_type_help: None,
            },
        )?;

    let fields: Vec<JsonField> = data_children
        .nodes()
        .iter()
        .filter(|&node| node.name().value() == FIELD_DEFINITOIN)
        .enumerate()
        .map(|(index, node)| {
            parse_field(
                node,
                source_code.clone(),
                name,
                &maybe_default_encoding,
                index,
            )
        })
        .collect::<Result<Vec<JsonField>, ParsingError>>()?;

    Ok(JsonDefinition {
        index,
        name: name.into(),
        doc: doc.into(),
        hash,
        fields,
    })
}

fn parse_field(
    node: &KdlNode,
    source_code: Arc<SourceInfo>,
    data_name: &str,
    maybe_default_encoding: &Option<TypeEncoding>,
    index: usize,
) -> Result<JsonField, ParsingError> {
    let field_node = node.extract_argument_string(
        0,
        ErrorContext {
            source_info: source_code.clone(),
            context: format!("field definition in JSON {data_name}").into(),
            not_found_help: Some("add a name to the field".into()),
            wrong_type_help: None,
        },
    )?;

    let maybe_encoding = node
        .get("encoding")
        .and_then(|v| v.as_string())
        .map(TypeEncoding::from_str)
        .transpose()
        .map_err(|e| {
            ParsingError::from(Diagnostic {
                message: e,
                severity: Severity::Warning,
                source_info: source_code.clone(),
                span: node.span(),
                help: None,
                label: None,
                related: vec![],
            })
        })?
        .or(*maybe_default_encoding);

    let datatype = {
        let datatype_entry =
            node.entry(TYPE_PROPERTY).ok_or_else(|| -> ParsingError {
                    ParsingError::from(Diagnostic {
                        message: format!(
                            "property `{TYPE_PROPERTY}` not provided for JSON field definition `{data_name}::{field_node}`",
                        ),
                        severity: Severity::Error,
                        source_info: source_code.clone(),
                        span: node.span(),
                        help: Some(format!("specify `{TYPE_PROPERTY}=\"...\"`.")),
                        label: None,
                        related: vec![],
                    })
        })?;

        let datatype_str= datatype_entry.value().as_string().ok_or_else(|| ParsingError::from(Diagnostic{
                message: format!(
                    "property `{TYPE_PROPERTY}`, of JSON field definition `{data_name}::{field_node}`, is not a string",
                ),
                severity: Severity::Error,
                source_info: source_code.clone(),
                span: datatype_entry.span(),
                help: Some(format!("specify `{TYPE_PROPERTY}=\"...\"`.")),
                label: None,
                related: vec![],
            }))?;

        generic_parse(
            datatype_str,
            maybe_encoding,
            source_code.clone(),
            datatype_entry.span(),
        )
    }?;

    let children = node.extract_children(ErrorContext {
        source_info: source_code.clone(),
        context: "field definition".into(),
        not_found_help: Some(format!("specify children `{KEY_CHILD}`, `{DOC_CHILD}`").into()),
        wrong_type_help: None,
    })?;

    let optional_node = children
        .get("optional")
        .map(|c| {
            c.extract_argument_bool(
                0,
                ErrorContext {
                    source_info: source_code.clone(),
                    context: format!("field definition `{data_name}::{field_node}`").into(),
                    not_found_help: None,
                    wrong_type_help: None,
                },
            )
        })
        .transpose()?
        .unwrap_or(false);

    let key_node = children.extract_child_node(
        "key",
        ErrorContext {
            source_info: source_code.clone(),
            context: format!("field definition `{data_name}::{field_node}`").into(),
            not_found_help: Some(
                format!(
                    "specify child `{KEY_CHILD} \"foobar\"` or `{KEY_CHILD} {TRANSPARENT_PROPERTY_NAME}=#true`"
                )
                .into(),
            ),
            wrong_type_help: None,
        },
    )?;

    let key = key_node
        // TODO(anri):
        // Generate a warning if `transparent` is used as an argument instead of
        // a property.
        // Or make such string reserved, or use another child, like `key-transparent`.
        .extract_argument_string(
            0,
            ErrorContext {
                source_info: source_code.clone(),
                context: format!("field definition `{data_name}::{field_node}`").into(),
                not_found_help: None,
                wrong_type_help: None,
            },
        )
        .map(|s| Some(JSONKey::String(s.to_owned())))
        .or_else(|_| {
            key_node
                .extract_property_bool(
                    TRANSPARENT_PROPERTY_NAME,
                    ErrorContext {
                        source_info: source_code.clone(),
                        context: format!("field definition `{data_name}::{field_node}`").into(),
                        not_found_help: None,
                        wrong_type_help: None,
                    },
                )
                // NOTE(anri):
                // Be _extremely_ careful when refactoring this code because after this method chain there
                // is a fancy and really cute ✨ unreachable! ✨ on `DataType` being `DataType::Custom`
                //
                // The check is done exactly here, but one may accidentally forget
                // about when refactoring and accidentally cause a `panic` later on.
                .and_then(|v| {
                    if !matches!(datatype, DataType::Custom(..)) {
                        return Err(ParsingError::from(Diagnostic {
                            message: format!("in field definition `{data_name}::{field_node}`, key `{TRANSPARENT_PROPERTY_NAME}` cannot be used for primitive types since they lack any underlying pre-defined key."),
                            severity: Severity::Error,
                            source_info: source_code.clone(),
                            span: key_node.span(),
                            help: Some(format!("provide an actual string for the key, for example: `{KEY_CHILD} \"foobar\"`")),
                            label: None,
                            related: vec![],
                        }));
                    }

                    if v {
                        Ok(Some(JSONKey::UseUnderlying))
                    } else {
                        Ok(None)
                    }
                })
        })?
        .ok_or_else(|| {
            let underlying_datatype = match &datatype {
                DataType::Custom(s) => s,
                _ => unreachable!("currently looking at `{KEY_CHILD} {TRANSPARENT_PROPERTY_NAME}` in a JSON definition; in particular if the pointeed datatype is _not_ `Custom`, then this error condition should never be triggered since the same check was done before"),
            };

        ParsingError::from(Diagnostic {
                message: format!("field `{KEY_CHILD}` in JSON definition `{data_name}::{field_node}` is ambigous; either specify a string for the key or `{TRANSPARENT_PROPERTY_NAME}=#true` to use the underlying key defined in `{underlying_datatype}::{HASH_CHILD}`."),
                severity: Severity::Error,
                source_info: source_code.clone(),
                span: key_node.span(),
                help: Some(format!("the child `{KEY_CHILD}` needs to be either a string or it must have the property `{TRANSPARENT_PROPERTY_NAME}=#true`, otherwise there is no way to know which key this field should use.")),
                label: None,
                related: vec![],
            })
        })?;

    let doc = children
        .extract_child_node(
            DOC_CHILD,
            ErrorContext {
                source_info: source_code.clone(),
                context: format!("field definition `{data_name}::{field_node}`").into(),
                not_found_help: Some(format!("specify child `{DOC_CHILD} \"Example\"`").into()),
                wrong_type_help: None,
            },
        )?
        .extract_argument_string(
            0,
            ErrorContext {
                source_info: source_code.clone(),
                context: format!("field definition `{data_name}::{field_node}`").into(),
                not_found_help: None,
                wrong_type_help: None,
            },
        )?;

    Ok(JsonField {
        index,
        name: field_node.to_owned(),
        r#type: datatype,
        key,
        doc: doc.to_owned(),
        escape: false,
        optional: optional_node,
    })
}
