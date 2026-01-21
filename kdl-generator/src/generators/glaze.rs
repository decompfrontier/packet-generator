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
            .filter_map(|def| match **def {
                Definition::Json(ref json) => Some(generate_json_cxx(registry, json)),
                _ => None,
            })
            .collect();

        match generated_sources {
            Ok(content) => {
                let inner = content.join("\n\n");

                let content = format!(
                    "// Glaze definitions
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
        }
        | DataType::F64 => Ok(format!("glz::quoted_num<&T::{name}>")),

        DataType::F32 {
            encoding: Encoding::Int,
        } => Ok(format!("glz::write_float32_t(&T::{name})")),
        DataType::I64 {
            encoding: Encoding::Int,
        } => Ok(format!("glz::write_float64_t(&T::{name})")),
        DataType::Bool {
            encoding: BoolEncoding::String,
        } => Ok(format!("pkg::glaze::bool_as_string<T, {name}>()")),
        DataType::Bool {
            encoding: BoolEncoding::Int,
        } => Ok(format!("glz::bools_as_numbers<&T::{name}>")),
        DataType::Datetime => Ok(format!("pkg::glaze::datetime<T, &T::{name}>()")),
        DataType::DatetimeUnix => Ok(format!("pkg::glaze::datetime_unix<T, &T::{name}>()")),
        DataType::StringArray {
            inner_type: _,
            separator,
        } => {
            let glz_sep = get_glz_array_separator(*separator);
            Ok(format!(
                "pkg::glaze::array_as_string_list<T, &T::{name}, '{glz_sep}'>"
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
) -> Result<String, GenerationError> {
    let struct_name = json.name.to_pascal_case();

    let fields: String = json
        .fields
        .iter()
        .map(|field| -> Result<String, GenerationError> {
            let mapper =
                get_glz_mapper(/*&struct_name,*/ &field.name, &field.type_, registry)?;
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
