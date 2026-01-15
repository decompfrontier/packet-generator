use itertools::Itertools;
use rootcause::Report;
use stringcase::Caser;

use crate::generators::GenerationError;

use crate::intermediate::{DataType, Definition, DefinitionRegistry, IntEnum, Json, StringEnum, Encoding, JSONKey};

const TAB: &str = "    ";

// TODO(arves): Move this to mod.rs / remove...
#[derive(Debug, Clone, Default)]
pub struct CxxSourceCode {
    pub filename: String,
    pub content: String,
}

/// Converts a DataType to types recognized by C++ with Glaze.
fn get_glz_mapper(
    parent_name: &str,
    name: &str,
    datatype: &DataType,
    registry: &DefinitionRegistry,
) -> Result<String, GenerationError> {
    match datatype {

        DataType::I32 { Encoding::String } |
        DataType::U32 { Encoding::String } |
        DataType::F32 { Encoding::String } |
        DataType::F64 { Encoding::String }
            => Ok(format!("glz::quoted_num<&T::{name}")),

        DataType::F32 { Encoding::Int } => Ok(format!("glzhlp::write_float32(&T::{name})")),
        DataType::I64 { Encoding::Int } => Ok(format!("glzhlp::write_float64(&T::{name})")),
        DataType::Bool { Encoding::String } => Ok(format!("glzhlp::strbool<{parent_name}, {name}>")),
        DataType::Bool { Encoding::Int } => Ok(format!("glz::bools_as_numbers<&T::{name}>")),
        DataType::Datetime => Ok(format!("glzhlp::datetime<{parent_name}, &T::{name}>")),
        DataType::DatetimeUnix => Ok(format!("glzhlp::datetimeunix<{parent_name}, &T::{name}>")),

        // TODO(arves): Boolean encoding without being int or string ??
        // TODO(arves): Quoted strings ?? (glz::quoted<&T::{name}>)
        // TODO(arves): Maps..
        // TODO(arves): String arrays (glzhlp::stringlist<{parent_name}, &T::{name}, '{character}'})
        // TODO(arves): Should single element array have a special mapping ??

        _ => Ok(format!("&T::{name}")),
    }
}


fn generate_json_cxx(
    registry: &DefinitionRegistry,
    json: &Json,
) -> Result<String, Report<GenerationError>> {
    let struct_name = json.name.to_pascal_case();

    let fields: String = json
        .fields
        .values()
        .map(|field| -> Result<String, GenerationError> {
            // TODO(arves): Fix parent name...
            let mapper = get_glz_mapper(&struct_name, &field.name, &field.type_, registry)?;
            let key: &str;
            match &field.key {
                JSONKey::String(v) => key = &v,
                _ => return Err(GenerationError::ExpiredRegistry {
                queried_from: field.type_.clone(),
            })
            }

            Ok(format!("{TAB}{TAB}\"{key}\", {mapper}"))
        })
        .process_results(|mut x| x.join(",\n"))?;


    let content = format!(
        r#"template <>
struct glz::meta<{struct_name}> {{
{TAB}using T = {struct_name};
{TAB}static constexpr auto value = object({fields}
{TAB});
}};"#,
    );

    Ok(content)
}

fn generate_int_enum_cxx(
    _registry: &DefinitionRegistry,
    int_enum: &IntEnum,
) -> Result<String, Report<GenerationError>> {
    // TODO(arves): implement
    Ok(String::new())
}

fn generate_str_enum_cxx(
    _registry: &DefinitionRegistry,
    str_enum: &StringEnum,
) -> Result<String, Report<GenerationError>> {
    // TODO(arves): implement
    Ok(String::new())
}


pub fn generate_glaze(
    registry: &DefinitionRegistry,
) -> Result<CxxSourceCode, Report<GenerationError>> {
    
    let generated_sources: Result<Vec<String>, Report<GenerationError>> = registry
        .definitions
        .values()
        .map(|def| match **def {
            Definition::Json(ref json) => generate_json_cxx(registry, json),
            Definition::Struct(ref json) => generate_json_cxx(registry, json),
            Definition::IntEnum(ref int_enum) => generate_int_enum_cxx(registry, int_enum),
            Definition::StringEnum(ref string_enum) => generate_str_enum_cxx(registry, string_enum),
        })
        .collect();

    let content = generated_sources?.join("\n\n");

    
    Ok(CxxSourceCode {
        filename: "test.hpp".to_owned(), // TODO(arves): test -> registry.name?
        content: format!(r#"#pragma once
#include <pkgen_glaze_helpers.hpp>

{content}"#)
    })
}
