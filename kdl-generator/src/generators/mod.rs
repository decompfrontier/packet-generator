use itertools::Itertools;
use rootcause::Report;
use stringcase::Caser;

use crate::intermediate::{DataType, Definition, DefinitionRegistry, IntEnum, Json, StringEnum};

const AUTOGENERATION_NOTICE: &str = r#"
// This file is auto-generated from a KDL specification by `packet-generator`.
// Please do not modify this, but instead change the original definitions."#;

const TAB: &str = "    ";

#[derive(Debug, Clone, Default)]
pub struct CxxSourceCode {
    pub filename: String,
    pub content: String,
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum GenerationError {
    #[error("whatever")]
    Whatever,

    #[error(
        "expired dependant type entry from type `{:#?}`; the registry may have been de-allocated",
        queried_from
    )]
    ExpiredRegistry { queried_from: DataType },

    #[error(
        "datatype `{queried_from:#?}` depended on type definition `{name}`, but the latter was not found"
    )]
    TypeNotFound {
        name: String,
        queried_from: DataType,
    },
}

pub fn generate_glaze(
    registry: &DefinitionRegistry,
) -> Result<Vec<CxxSourceCode>, Report<GenerationError>> {
    let generated_sources: Result<Vec<CxxSourceCode>, Report<GenerationError>> = registry
        .definitions
        .values()
        .map(|def| match **def {
            Definition::Json(ref json) => generate_json_cxx(registry, json),
            Definition::Struct(ref json) => generate_json_cxx(registry, json),
            Definition::IntEnum(ref int_enum) => generate_int_enum_cxx(registry, int_enum),
            Definition::StringEnum(ref string_enum) => generate_str_enum_cxx(registry, string_enum),
        })
        .collect();

    generated_sources
}

/// Converts a DataType to types recognized by C++ with Glaze.
fn convert_datatype(
    datatype: &DataType,
    registry: &DefinitionRegistry,
) -> Result<String, GenerationError> {
    match datatype {
        DataType::I32 { .. } => Ok(String::from("int")),

        DataType::U32 { .. } => Ok(String::from("unsigned int")),

        DataType::I64 { .. } => Ok(String::from("long")),

        DataType::U64 { .. } => Ok(String::from("unsigned long")),

        DataType::F32 { .. } => Ok(String::from("float")),

        DataType::F64 => Ok(String::from("double")),

        DataType::Bool { .. } => Ok(String::from("bool")),

        DataType::String => Ok(String::from("std::string")),

        DataType::Datetime => Ok(String::from("glzhlp::chronotime")),

        DataType::DatetimeUnix => Ok(String::from("glzhlp::chronotime")),

        DataType::Map { key, value } => {
            let key = convert_datatype(key, registry)?;
            let value = convert_datatype(value, registry)?;

            Ok(format!("std::unordered_map<{key}, {value}>"))
        }

        DataType::StringArray {
            inner_type: _,
            separator: _,
        } => Ok(String::from("std::string")),

        DataType::Array { inner_type } => {
            let inner = convert_datatype(inner_type, registry)?;

            Ok(format!("std::vector<{inner}>"))
        }

        DataType::SingleElementArray { inner_type } => {
            let inner = convert_datatype(inner_type, registry)?;

            Ok(format!("std::array<{inner}, 1>"))
        }

        DataType::Definition(weak) => match weak.upgrade() {
            Some(definition) => Ok(definition.name().clone()),
            None => Err(GenerationError::ExpiredRegistry {
                queried_from: datatype.clone(),
            }),
        },

        DataType::Unknown(other) => match registry.find(other) {
            Some(definition) => Ok(definition.name().clone()),
            None => Err(GenerationError::TypeNotFound {
                name: other.to_string(),
                queried_from: datatype.clone(),
            }),
        },
    }
}

fn generate_json_cxx(
    registry: &DefinitionRegistry,
    json: &Json,
) -> Result<CxxSourceCode, Report<GenerationError>> {
    let filename = format!("{}.h", json.name);

    // TODO(anri):
    // Calculate the approximate sizes of the C++ types and re-order the fields
    //   to pack them more efficiently, from largest to smallest.
    // We could do this optimization because JSON has no ordering requirement.

    let fields: String = json
        .fields
        .values()
        .map(|field| -> Result<String, GenerationError> {
            let datatype = convert_datatype(&field.type_, registry)?;
            let name = field.name.to_snake_case();

            Ok(format!("{TAB}{datatype} {name};"))
        })
        .process_results(|mut x| x.join("\n"))?;

    let struct_name = json.name.to_pascal_case();

    let content = format!(
        r#"#pragma once

{AUTOGENERATION_NOTICE}

struct {struct_name} {{
{fields}
}};"#,
    );

    Ok(CxxSourceCode { filename, content })
}

fn generate_int_enum_cxx(
    _registry: &DefinitionRegistry,
    int_enum: &IntEnum,
) -> Result<CxxSourceCode, Report<GenerationError>> {
    let filename = format!("{}.h", int_enum.name.to_pascal_case());
    let start = int_enum.start;

    let mut variants_iter = int_enum
        .variants
        .values()
        .sorted_unstable_by_key(|a| a.index);

    let first_variant = variants_iter
        .next()
        .map(|variant| {
            let name = variant.name.to_pascal_case();

            let start = variant.value.unwrap_or(start);
            let doc = &variant.doc;

            format!("\n{TAB}/// {doc}\n{TAB}{name} = {start},")
        })
        .unwrap_or_default();

    let variants_str = variants_iter
        .map(|variant| {
            let name = variant.name.to_pascal_case();
            let maybe_val = variant.value.map(|v| format!(" = {v}")).unwrap_or_default();
            let doc = &variant.doc;

            format!("\n{TAB}/// {doc}\n{TAB}{name}{maybe_val}")
        })
        .join(",\n");

    let doc = &int_enum.doc;

    let content = format!(
        r#"#pragma once

{AUTOGENERATION_NOTICE}

/// {doc}
enum class {} {{
{first_variant}
{variants_str}
}};"#,
        int_enum.name.to_pascal_case(),
    );

    Ok(CxxSourceCode { filename, content })
}

fn generate_str_enum_cxx(
    _registry: &DefinitionRegistry,
    str_enum: &StringEnum,
) -> Result<CxxSourceCode, Report<GenerationError>> {
    let filename = format!("{}.h", str_enum.name.to_pascal_case());

    let variants = str_enum
        .variants
        .values()
        .sorted_unstable_by_key(|a| a.index)
        .map(|variant| {
            let name = variant.name.to_pascal_case();
            let doc = &variant.doc;
            format!("\n{TAB}/// {doc}\n{TAB}{name}")
        })
        .join(",\n");

    let doc = &str_enum.doc;

    let content = format!(
        r#"#pragma once

{AUTOGENERATION_NOTICE}

/// {doc}
enum class {} {{
{variants}
}};"#,
        str_enum.name.to_pascal_case(),
    );

    Ok(CxxSourceCode { filename, content })
}
