use itertools::Itertools;
use stringcase::Caser;

use crate::generators::{Addon, CxxGenerator, GenerationError};

use crate::intermediate::{
    ArraySeparator, BoolEncoding, DataType, Definition, DefinitionRegistry, Encoding, Json,
};

const TAB: &str = "    ";

#[derive(Debug, Clone)]
pub struct GlazeGenerator;

impl Addon for GlazeGenerator {
    type For = CxxGenerator;

    fn content(
        &self,
        registry: &DefinitionRegistry,
    ) -> Option<Result<std::borrow::Cow<'static, str>, GenerationError>> {
        let generated_sources: Result<Vec<String>, GenerationError> = registry
            .all_definitions()
            .filter_map(|def| match registry.get(def) {
                Definition::Json(json) => Some(generate_json_cxx(registry, json)),
                _ => None,
            })
            .collect();

        match generated_sources {
            Ok(content) => {
                let inner = content.join("\n\n");

                let content = format!(
                    "// Auto-generated Glaze definitions
#if __has_include(<glaze/glaze.hpp>)
#include <pkgen_glaze_helpers.hpp>

{inner}

#endif // __has_include(<glaze/glaze.hpp>)
        "
                );

                Some(Ok(content.into()))
            }
            Err(e) => Some(Err(e)),
        }
    }
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
    //parent_name: &str,
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
        } => Ok(format!("glz::quoted_num<&T::{name}>")),

        DataType::F32 {
            encoding: Encoding::Int,
        } => Ok(format!("glz_write_f32(&T::{name})")),
        DataType::F64 => Ok(format!("glz_write_f64(&T::{name})")),
        DataType::Bool {
            encoding: BoolEncoding::String,
        } => Ok(format!("pkg::glaze::bool_as_string<&T::{name}>()")),
        DataType::Bool {
            encoding: BoolEncoding::Int,
        } => Ok(format!("glz::bools_as_numbers<&T::{name}>")),
        DataType::Datetime => Ok(format!("pkg::glaze::datetime<&T::{name}>()")),
        DataType::DatetimeUnix => Ok(format!("pkg::glaze::datetime_unix<&T::{name}>()")),
        DataType::SingleElementArray { inner_type: _ } => {
            Ok(format!("pkg::glaze::single_array<&T::{name}>()"))
        }
        DataType::StringArray {
            inner_type: _,
            separator,
        } => {
            let glz_sep = get_glz_array_separator(*separator);
            Ok(format!(
                "pkg::glaze::array_string<T, &T::{name}, '{glz_sep}'>"
            ))
        }

        // NOTE(arves) => For custom encoding one should override the specific concepts
        _ => Ok(format!("&T::{name}")),
    }
}

fn generate_json_cxx(
    registry: &DefinitionRegistry,
    json: &Json,
) -> Result<String, GenerationError> {
    let struct_name = json.name.to_pascal_case();

    let fields: String = json
        .fields
        .iter()
        .map(|field| -> Result<String, GenerationError> {
            let mapper =
                get_glz_mapper(/* &struct_name, */ &field.name, &field.type_, registry)?;
            let key: &str = &field.key;

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
