use itertools::Itertools;
use rootcause::Report;
use stringcase::Caser;

use crate::generators::GenerationError;

use crate::intermediate::{
    ArraySeparator, BoolEncoding, DataType, Definition, DefinitionRegistry, Encoding, JSONKey, Json,
};

const TAB: &str = "    ";

// TODO(arves): Move this to mod.rs / remove...
#[derive(Debug, Clone, Default)]
pub struct CxxSourceCode {
    pub filename: String,
    pub content: String,
}

const fn get_glz_array_separator(sep: ArraySeparator) -> char {
    match sep {
        ArraySeparator::Comma => ',',
        ArraySeparator::At => '@',
        ArraySeparator::Colon => ':',
    }
}

/// Converts a `DataType` to types recognized by C++ with Glaze.
fn get_glz_mapper(
    parent_name: &str,
    name: &str,
    datatype: &DataType,
    _registry: &DefinitionRegistry,
) -> Result<String, GenerationError> {
    let name = name.to_snake_case();

    match datatype {
        DataType::I32 {
            encoding: Encoding::String,
        }
        | DataType::U32 {
            encoding: Encoding::String,
        }
        | DataType::F32 {
            encoding: Encoding::String,
        }
        | DataType::F64 => Ok(format!("glz::quoted_num<&T::{name}>")),

        DataType::F32 {
            encoding: Encoding::Int,
        } => Ok(format!("glzhlp::write_float32(&T::{name})")),
        DataType::I64 {
            encoding: Encoding::Int,
        } => Ok(format!("glzhlp::write_float64(&T::{name})")),
        DataType::Bool {
            encoding: BoolEncoding::String,
        } => Ok(format!("glzhlp::strbool<{parent_name}, {name}>")),
        DataType::Bool {
            encoding: BoolEncoding::Int,
        } => Ok(format!("glz::bools_as_numbers<&T::{name}>")),
        DataType::Datetime => Ok(format!("glzhlp::datetime<{parent_name}, &T::{name}>")),
        DataType::DatetimeUnix => Ok(format!("glzhlp::datetimeunix<{parent_name}, &T::{name}>")),
        DataType::String => Ok(format!("glz::quoted<&T::{name}>")),
        DataType::StringArray {
            inner_type: _,
            separator,
        } => {
            let glz_sep = get_glz_array_separator(*separator);
            Ok(format!(
                "glzhlp::stringlist<{parent_name}, &T::{name}, '{glz_sep}'>"
            ))
        }

        // NOTE(arves): Investigate if single array needs a custom mapping if we do not explicitally declare them as "std::array"

        // TODO(arves): Using custom encoding on vector or maps WILL NOT WORK! Find a way to fix it in glaze (if it's possible)
        _ => Ok(format!("&T::{name}")),
    }
}

fn generate_json_cxx(
    registry: &DefinitionRegistry,
    json: &Json,
) -> Result<String, Report<GenerationError>> {
    let struct_name = json.name.to_pascal_case(); // TODO(arves): Does not handle the array type blaaah

    let fields: String = json
        .fields
        .iter()
        .map(|field| -> Result<String, GenerationError> {
            // TODO(arves): Fix parent name...
            let mapper = get_glz_mapper(&struct_name, &field.name, &field.type_, registry)?;
            let key: &str = match &field.key {
                JSONKey::String(v) => v,

                JSONKey::UseUnderlying => {

                    // TODO(anri):
                    // Add validation in transformation between RawDocument -> Document,
                    // then restructure all these assertions/panics as errors.
                    match &field.type_ {
                        DataType::Definition(def) => {
                            let Some(def) = def.upgrade() else {
                                return Err(GenerationError::ExpiredRegistry { queried_from: field.type_.clone() });
                            };

                            match *def {
                                Definition::Json(ref json) => {
                                    &json.hash_name.clone().unwrap_or_else(|| panic!("the parser should've checked that {} contains a `hash`", json.name))
                                }

                                _ => todo!(),
                            }
                        }

                        _ => todo!("In glaze.rs generation, recursively handle `JSONKey::UseUnderlying` for types which are not directly a definition. Should be an error.")
                    }
                }
            };

            Ok(format!("{TAB}{TAB}\"{key}\", {mapper}"))
        })
        .process_results(|mut x| x.join(",\n"))?;

    let content = format!(
        "template <>
struct glz::meta<{struct_name}> {{
{TAB}using T = {struct_name};
{TAB}static constexpr auto value = object(
{fields}
{TAB});
}};"
    );

    Ok(content)
}

pub fn generate_glaze(
    registry: &DefinitionRegistry,
) -> Result<CxxSourceCode, Report<GenerationError>> {
    let generated_sources: Result<Vec<String>, Report<GenerationError>> = registry
        .all_definitions()
        .filter_map(|def| match **def {
            Definition::Json(ref json) => Some(generate_json_cxx(registry, json)),
            _ => None,
        })
        .collect();

    let content = generated_sources?.join("\n\n");

    Ok(CxxSourceCode {
        filename: "test.hpp".to_owned(), // TODO(arves): test -> registry.name?
        content: format!(
            "#pragma once
#include <pkgen_glaze_helpers.hpp>

{content}"
        ),
    })
}
