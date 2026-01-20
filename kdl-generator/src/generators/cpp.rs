use itertools::Itertools;
use rootcause::Report;
use stringcase::Caser;

use crate::generators::{GenerationError, PrimaryGenerator, SecondaryGenerator};

use crate::intermediate::{DataType, Definition, DefinitionRegistry, IntEnum, Json, StringEnum};

const AUTOGENERATION_NOTICE: &str = "
// This file is auto-generated from a KDL specification by `packet-generator`.
// Please do not modify this, but instead change the original definitions.";

const TAB: &str = "    ";

pub struct CxxGenerator;

/// Converts a `DataType` to types recognized by C++.
fn convert_datatype(
    datatype: &DataType,
    registry: &DefinitionRegistry,
) -> Result<String, GenerationError> {
    match datatype {
        DataType::I32 { .. } => Ok(String::from("int32_t")),

        DataType::U32 { .. } => Ok(String::from("uint32_t")),

        DataType::I64 { .. } => Ok(String::from("int64_t")),

        DataType::U64 { .. } => Ok(String::from("uint64_t")),

        DataType::F32 { .. } => Ok(String::from("pkg::float32")), // for supporting C++20 floating point

        DataType::F64 => Ok(String::from("pkg::float64")), // for supporting C++20 floating point

        DataType::Bool { .. } => Ok(String::from("bool")),

        DataType::String
        | DataType::StringArray {
            inner_type: _,
            separator: _,
        } => Ok(String::from("std::string")),

        DataType::Datetime | DataType::DatetimeUnix => Ok(String::from("pkg::chrono_time")),

        DataType::Map { key, value } => {
            let key = convert_datatype(key, registry)?;
            let value = convert_datatype(value, registry)?;

            Ok(format!("std::unordered_map<{key}, {value}>"))
        }

        DataType::Array { inner_type } => {
            let inner = convert_datatype(inner_type, registry)?;

            Ok(format!("std::vector<{inner}>"))
        }

        DataType::SingleElementArray { inner_type } => {
            let inner = convert_datatype(inner_type, registry)?;

            Ok(format!("std::array<{inner}, 1>")) // TODO(arves): Can this be made a meta-data only generation step?
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
) -> Result<String, Report<GenerationError>> {
    // TODO(anri):
    // Calculate the approximate sizes of the C++ types and re-order the fields
    //   to pack them more efficiently, from largest to smallest.
    // We could do this optimization because JSON has no ordering requirement.

    let fields: String = json
        .fields
        .iter()
        .map(|field| -> Result<String, GenerationError> {
            let datatype = convert_datatype(&field.type_, registry)?;
            let name = field.name.to_snake_case();

            Ok(format!("{TAB}{datatype} {name};"))
        })
        .process_results(|mut x| x.join("\n"))?;

    let struct_name = json.name.to_pascal_case();

    let content = format!(
        "struct {struct_name} {{
{fields}
}};"
    );

    Ok(content)
}

fn generate_int_enum_cxx(
    _registry: &DefinitionRegistry,
    int_enum: &IntEnum,
) -> Result<String, Report<GenerationError>> {
    let start = int_enum.start;

    let mut variants_iter = int_enum.variants.iter().sorted_unstable_by_key(|a| a.index);

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
        "/// {doc}
enum class {} {{
{first_variant}
{variants_str}
}};",
        int_enum.name.to_pascal_case(),
    );

    Ok(content)
}

fn generate_str_enum_cxx(
    _registry: &DefinitionRegistry,
    str_enum: &StringEnum,
) -> Result<String, Report<GenerationError>> {
    let variants = str_enum
        .variants
        .iter()
        .sorted_unstable_by_key(|a| a.index)
        .map(|variant| {
            let name = variant.name.to_pascal_case();
            let doc = &variant.doc;
            let val = &variant.value;
            format!("\n{TAB}/// {doc}\n{TAB}constexpr const auto {name} = \"{val}\";")
        })
        .join("\n");

    let doc = &str_enum.doc;

    let content = format!(
        "/// {doc}
namespace {} {{
{TAB}using Type = std::string;
{variants}
}};",
        str_enum.name.to_pascal_case(),
    );

    Ok(content)
}

impl PrimaryGenerator for CxxGenerator {
    fn get_output_file_name(&self, name: &str) -> String {
        format!("{name}.hpp")
    }
}

impl SecondaryGenerator for CxxGenerator {
    fn get_prefix(&self) -> String {
        format!(r#"#pragma once
#include <pkgen_helpers.hpp>

{AUTOGENERATION_NOTICE}
"#)
    }

    fn step(&self,
        registry: &DefinitionRegistry
    ) -> Result<String, Report<GenerationError>>
    {
        let generated_sources: Result<Vec<String>, Report<GenerationError>> = registry
            .all_definitions()
            .map(|def| match **def {
                Definition::Json(ref json) => generate_json_cxx(registry, json),
                Definition::IntEnum(ref int_enum) => generate_int_enum_cxx(registry, int_enum),
                Definition::StringEnum(ref string_enum) => generate_str_enum_cxx(registry, string_enum),
            })
            .collect();

        let content = generated_sources?.join("\n\n");
        Ok(content)
    }
}
