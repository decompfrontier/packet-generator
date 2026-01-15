use std::sync::Arc;

use kdl::KdlNode;
use miette::Severity;

use crate::kdl_parser::{
    Diagnostic, ParsingError, SourceInfo,
    parser::{ErrorContext, KdlDocumentUtilsExt, KdlNodeUtilsExt},
    schema::{IntEnumDefinition, IntEnumInner, StringEnumDefinition, StringEnumInner},
};

const ENUM_VARIANT_FIELD_NAME: &str = "variant";

pub fn parse_int_enum_definition(
    definition: &KdlNode,
    source_code: SourceInfo,
) -> Result<IntEnumDefinition, ParsingError> {
    let name = definition.extract_argument_string(
        0,
        ErrorContext {
            source_info: source_code.clone(),
            context: "definition".into(),
            not_found_help: Some("add a name to the enum definition".into()),
            wrong_type_help: Some("give it a name as a string".into()),
        },
    )?;

    let start = definition.extract_property_int(
        "start",
        ErrorContext {
            source_info: source_code.clone(),
            context: "int enum definition".into(),
            not_found_help: Some(
                "specify a `start` property to the int enum, for example `start=0`".into(),
            ),
            wrong_type_help: Some(
                "the type inside the `start` property must be an integer, for exaple `start=0`"
                    .into(),
            ),
        },
    )?;

    let data_children = definition.children().ok_or_else(|| {
        ParsingError::from(Diagnostic {
            message: "integer enum definition has no children".to_owned(),
            severity: Severity::Error,
            source_info: source_code.clone(),
            span: definition.span(),
            help: Some("specify children for the variants".to_owned()),
            label: None,
            related: vec![],
        })
    })?;

    let doc = data_children
        .extract_child_node(
            "doc",
            ErrorContext {
                source_info: source_code.clone(),
                context: format!("integer enum definition `{name}`").into(),
                not_found_help: Some("specify child `doc \"Example\"`".into()),
                wrong_type_help: None,
            },
        )?
        .extract_argument_string(
            0,
            ErrorContext {
                source_info: source_code.clone(),
                context: format!("integer enum definition `{name}`").into(),
                not_found_help: Some("specify child `doc \"Example\"`".into()),
                wrong_type_help: None,
            },
        )?;

    let variants: Vec<IntEnumInner> = data_children
        .nodes()
        .iter()
        .filter(|&node| node.name().value() == ENUM_VARIANT_FIELD_NAME)
        .enumerate()
        .map(|(index, node)| parse_int_enum_variant(node, source_code.clone(), name, index))
        .collect::<Result<Vec<_>, ParsingError>>()?;

    Ok(IntEnumDefinition {
        name: name.into(),
        doc: doc.into(),
        start,
        variants,
    })
}

fn parse_int_enum_variant(
    node: &KdlNode,
    source_code: SourceInfo,
    enum_name: &str,
    index: usize,
) -> Result<IntEnumInner, ParsingError> {
    const VALUE_PROPERTY: &str = "value";

    let name = node.extract_argument_string(
        0,
        ErrorContext {
            source_info: source_code.clone(),
            context: format!("variant definition in integer enum `{enum_name}`").into(),
            not_found_help: Some(
                format!("specify a name for the variant in integer enum `{enum_name}`").into(),
            ),
            wrong_type_help: None,
        },
    )?;

    let value = node
        .entry(VALUE_PROPERTY)
        .map(|entry| {
            entry.value().as_integer().ok_or_else(|| ParsingError::from(Diagnostic {
                message: format!("property `{VALUE_PROPERTY}` in integer enum variant `{enum_name}::{name}` is not an integer"),
                severity: Severity::Error,
                source_info: source_code.clone(),
                span: entry.span(),
                help: Some("the property `value` specifies the integer value of the enum variant, it can only be an integer.".to_owned()),
                label: None,
                related: vec![],
            }))
        })
        .transpose()?;

    let children = node.extract_children(ErrorContext {
        source_info: source_code.clone(),
        context: format!("integer enum variant `{enum_name}::{name}`").into(),
        not_found_help: Some("specify a child `doc \"Example\"`.".into()),
        wrong_type_help: None,
    })?;

    let doc = children
        .extract_child_node(
            "doc",
            ErrorContext {
                source_info: source_code.clone(),
                context: format!("integer enum variant definition `{enum_name}::{name}`").into(),
                not_found_help: Some("specify a child `doc \"Example\"`.".into()),
                wrong_type_help: Some("specify a child `doc \"Example\"`.".into()),
            },
        )?
        .extract_argument_string(
            0,
            ErrorContext {
                source_info: source_code,
                context: format!(
                    "child `doc` in integer enum variant definition `{enum_name}::{name}``"
                )
                .into(),
                not_found_help: Some("specify a child `doc \"Example\"`.".into()),
                wrong_type_help: Some("the child `doc` must be a string.".into()),
            },
        )?;

    Ok(IntEnumInner {
        name: name.to_owned(),
        index,
        value,
        doc: doc.to_owned(),
    })
}

pub fn parse_string_enum_definition(
    definition: &KdlNode,
    source_code: SourceInfo,
) -> Result<StringEnumDefinition, ParsingError> {
    let name = definition.extract_argument_string(
        0,
        ErrorContext {
            source_info: source_code.clone(),
            context: "definition".into(),
            not_found_help: Some("add a name to the enum definition".into()),
            wrong_type_help: Some("give it a name as a string".into()),
        },
    )?;

    let data_children = definition.children().ok_or_else(|| {
        ParsingError::from(Diagnostic {
            message: "string enum definition has no children".to_owned(),
            severity: Severity::Error,
            source_info: source_code.clone(),
            span: definition.span(),
            help: Some("specify children for the enum".to_owned()),
            label: None,
            related: vec![],
        })
    })?;

    let doc = data_children
        .extract_child_node(
            "doc",
            ErrorContext {
                source_info: source_code.clone(),
                context: format!("string enum definition `{name}`").into(),
                not_found_help: Some("specify child `doc \"Example\"`".into()),
                wrong_type_help: None,
            },
        )?
        .extract_argument_string(
            0,
            ErrorContext {
                source_info: source_code.clone(),
                context: format!("string enum definition `{name}`").into(),
                not_found_help: Some("specify child `doc \"Example\"`".into()),
                wrong_type_help: None,
            },
        )?;

    let variants: Vec<StringEnumInner> = data_children
        .nodes()
        .iter()
        .filter(|&node| node.name().value() == ENUM_VARIANT_FIELD_NAME)
        .enumerate()
        .map(|(index, node)| parse_string_enum_variant(node, source_code.clone(), name, index))
        .collect::<Result<Vec<_>, ParsingError>>()?;

    Ok(StringEnumDefinition {
        name: name.into(),
        doc: doc.into(),
        variants,
    })
}

fn parse_string_enum_variant(
    node: &KdlNode,
    source_code: SourceInfo,
    enum_name: &str,
    index: usize,
) -> Result<StringEnumInner, ParsingError> {
    // const VALUE_PROPERTY: &str = "value";

    let name = node.extract_argument_string(
        0,
        ErrorContext {
            source_info: source_code.clone(),
            context: format!("variant definition in string enum `{enum_name}`").into(),
            not_found_help: Some(
                format!("specify a name for the variant in string enum `{enum_name}`").into(),
            ),
            wrong_type_help: None,
        },
    )?;

    // let value = node
    //     .get(VALUE_PROPERTY)
    //     .map(|x| {
    //         x.as_string().ok_or_else(|| ParsingError::from(Diagnostic {
    //             message: format!("property `{VALUE_PROPERTY}` in string enum variant `{enum_name}::{name}` is not an integer"),
    //             severity: Severity::Error,
    //             source_code: source_code.clone(),
    //             span: node.span(),
    //             help: Some("the property `value` specifies the string value of the enum variant, it can only be a string.".to_owned()),
    //             label: None,
    //             related: vec![],
    //         }))
    //     })
    //     .transpose()?;

    let children = node.extract_children(ErrorContext {
        source_info: source_code.clone(),
        context: format!("integer enum variant `{enum_name}::{name}`").into(),
        not_found_help: Some("specify a child `doc \"Example\"`.".into()),
        wrong_type_help: None,
    })?;

    let doc = children
        .extract_child_node(
            "doc",
            ErrorContext {
                source_info: source_code.clone(),
                context: format!("integer enum variant definition `{enum_name}::{name}`").into(),
                not_found_help: Some("specify a child `doc \"Example\"`.".into()),
                wrong_type_help: Some("specify a child `doc \"Example\"`.".into()),
            },
        )?
        .extract_argument_string(
            0,
            ErrorContext {
                source_info: source_code,
                context: format!(
                    "child `doc` in string enum variant definition `{enum_name}::{name}``"
                )
                .into(),
                not_found_help: Some("specify a child `doc \"Example\"`.".into()),
                wrong_type_help: Some("the child `doc` must be a string.".into()),
            },
        )?;

    Ok(StringEnumInner {
        name: name.to_owned(),
        index,
        value: None,
        doc: doc.to_owned(),
    })
}
