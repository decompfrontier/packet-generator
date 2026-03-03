use std::path::PathBuf;

use atomicow::CowArc;
use itertools::Itertools;
use stringcase::Caser;

use crate::generators::{Addon, GeneratedSource, GenerationError, Generator, WithAddons};

use crate::intermediate::{
    DataType, Definition, DefinitionRegistry, IntEnum, Json, JsonField, StringEnum,
};

const AUTOGENERATION_NOTICE: &str = "
// This file is auto-generated from a KDL specification by `packet-generator`.
// Please do not modify this, but instead change the original definitions.";

const TAB: &str = "    ";

fn split_documentation(doc: &str, indent_level: usize) -> String {
    super::utils::split_documentation(doc, TAB, "///", indent_level)
}

fn struct_format(string: &str) -> String {
    string.to_pascal_case()
}

fn enum_format(string: &str) -> String {
    string.to_pascal_case()
}

fn enum_variant_format(string: &str) -> String {
    string.to_pascal_case()
}

fn field_format(string: &str) -> String {
    string.to_snake_case()
}

#[derive(Debug)]
pub struct CxxGenerator {
    addons: Vec<Box<dyn Addon<For = Self>>>,
    _private: (),
}

impl Default for CxxGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl CxxGenerator {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            addons: vec![],
            _private: (),
        }
    }
}

impl Generator for CxxGenerator {
    fn generate(
        &self,
        registry: &DefinitionRegistry,
        initial_filename: &str,
    ) -> Result<Vec<GeneratedSource>, GenerationError> {
        let mut content = format!(
            r#"#pragma once

#include <pkgen_helpers.hpp>

{AUTOGENERATION_NOTICE}

"#
        );

        for addon in &self.addons {
            if let Some(preamble) = addon.preamble(registry) {
                content.push_str(&preamble);
                content.push_str("\n\n");
            }
        }

        let forward_definitions: Result<Vec<String>, _> = registry
            .all_definitions()
            .map(|def| match registry.get(def) {
                Definition::Json(json) => Ok(format!("struct {};", json.name)),
                Definition::IntEnum(int_enum) => Ok(format!("enum class {};", int_enum.name)),
                Definition::StringEnum(string_enum) => generate_str_enum_cxx(registry, string_enum),
            })
            .collect();

        content.push_str(&forward_definitions?.join("\n\n"));
        content.push_str("\n\n");

        let generated_sources: Result<Vec<String>, _> = registry
            .sorted_definitions()
            .map_err(GenerationError::CycleFound)?
            .iter()
            .filter_map(|&def| match registry.get(def) {
                Definition::Json(json) => Some(generate_json_cxx(registry, json)),
                Definition::IntEnum(int_enum) => Some(generate_int_enum_cxx(registry, int_enum)),
                Definition::StringEnum(_string_enum) => None,
            })
            .collect();

        content.push_str(&generated_sources?.join("\n\n"));
        content.push_str("\n\n");

        for addon in &self.addons {
            if let Some(addon_content) = addon.content(registry) {
                content.push_str(&addon_content?);
                content.push('\n');
            }
        }

        for addon in &self.addons {
            if let Some(postamble) = addon.postamble(registry) {
                content.push_str(&postamble);
                content.push_str("\n\n");
            }
        }

        Ok(vec![GeneratedSource {
            filename: PathBuf::from(format!("{}.hpp", initial_filename)),
            content,
        }])
    }

    fn json_name<'a>(&'a self, definition: &'a Json) -> CowArc<'a, str> {
        CowArc::Owned(struct_format(&definition.name).into())
    }

    fn json_field_name<'a>(&'a self, definition: &'a JsonField) -> CowArc<'a, str> {
        CowArc::Owned(field_format(&definition.name).into())
    }

    fn int_enum_name<'a>(&'a self, definition: &'a IntEnum) -> CowArc<'a, str> {
        CowArc::Owned(enum_format(&definition.name).into())
    }

    fn int_enum_variant_name<'a>(
        &'a self,
        definition: &'a crate::intermediate::IntEnumVariant,
    ) -> CowArc<'a, str> {
        CowArc::Owned(enum_variant_format(&definition.name).into())
    }

    fn string_enum_name<'a>(&'a self, definition: &'a StringEnum) -> CowArc<'a, str> {
        CowArc::Owned(enum_format(&definition.name).into())
    }

    fn string_enum_variant_name<'a>(
        &'a self,
        definition: &'a crate::intermediate::StringEnumVariant,
    ) -> CowArc<'a, str> {
        CowArc::Owned(enum_variant_format(&definition.name).into())
    }
}

impl WithAddons for CxxGenerator {
    fn add_addon<T>(&mut self, addon: T)
    where
        T: super::Addon<For = Self> + 'static,
        Self: Sized,
    {
        self.addons.push(Box::new(addon));
    }
}

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

        DataType::String => Ok(String::from("std::string")),

        DataType::StringArray {
            inner_type,
            separator: _,
        } => {
            let inner = convert_datatype(inner_type, registry)?;

            Ok(format!("pkg::string_list<{inner}>"))
        }

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
            Ok(inner) // TODO(arves): Can this be made a meta-data only generation step?
        }

        DataType::Definition(weak) => {
            let definition = registry.get(*weak);
            match *definition {
                Definition::StringEnum(ref str_enum) => Ok(format!("{}::Type", str_enum.name)),

                _ => Ok(definition.name().clone()),
            }
        }

        DataType::Unknown(other) => match registry.find(other) {
            Some((definition, _idx)) => match definition {
                Definition::StringEnum(str_enum) => Ok(format!("{}::Type", str_enum.name)),

                _ => Ok(definition.name().clone()),
            },

            None => Err(GenerationError::TypeNotFound {
                name: other.clone(),
                queried_from: datatype.clone(),
            }),
        },
    }
}

fn generate_json_cxx(
    registry: &DefinitionRegistry,
    json: &Json,
) -> Result<String, GenerationError> {
    // TODO(anri):
    // Calculate the approximate sizes of the C++ types and re-order the fields
    //   to pack them more efficiently, from largest to smallest.
    // We could do this optimization because JSON has no ordering requirement.

    let fields: String = json
        .fields
        .iter()
        .map(|field| -> Result<String, GenerationError> {
            let datatype = convert_datatype(&field.type_, registry)?;
            let name = field_format(&field.name);
            let doc = split_documentation(&field.doc, 0);

            Ok(format!(
                "
{TAB}{doc}
{TAB}{datatype} {name};"
            ))
        })
        .process_results(|mut x| x.join("\n"))?;

    let struct_name = struct_format(&json.name);
    let struct_doc = split_documentation(&json.doc, 0);

    let content = format!(
        "
{struct_doc}
struct {struct_name} {{
{fields}
}};"
    );

    Ok(content)
}

fn generate_int_enum_cxx(
    _registry: &DefinitionRegistry,
    int_enum: &IntEnum,
) -> Result<String, GenerationError> {
    let start = int_enum.start;

    let mut variants_iter = int_enum.variants.iter().sorted_unstable_by_key(|a| a.index);

    let first_variant = variants_iter
        .next()
        .map(|variant| {
            let name = enum_variant_format(&variant.name);

            let start = variant.value.unwrap_or(start);
            let doc = &variant.doc;

            format!("\n{TAB}/// {doc}\n{TAB}{name} = {start},")
        })
        .unwrap_or_default();

    let variants_str = variants_iter
        .map(|variant| {
            let name = enum_variant_format(&variant.name);
            let maybe_val = variant.value.map(|v| format!(" = {v}")).unwrap_or_default();
            let doc = &variant.doc;

            format!("\n{TAB}/// {doc}\n{TAB}{name}{maybe_val}")
        })
        .join(",\n");

    let doc = split_documentation(&int_enum.doc, 0);

    let content = format!(
        "{doc}
enum class {} {{
{first_variant}
{variants_str}
}};",
        enum_format(&int_enum.name),
    );

    Ok(content)
}

fn generate_str_enum_cxx(
    _registry: &DefinitionRegistry,
    str_enum: &StringEnum,
) -> Result<String, GenerationError> {
    let variants = str_enum
        .variants
        .iter()
        .sorted_unstable_by_key(|a| a.index)
        .map(|variant| {
            let name = enum_variant_format(&variant.name);
            let doc = &variant.doc;
            let val = &variant.value;
            format!("\n{TAB}/// {doc}\n{TAB}constexpr const auto {name} = \"{val}\";")
        })
        .join("\n");

    let doc = split_documentation(&str_enum.doc, 0);

    let content = format!(
        "{doc}
namespace {} {{
{TAB}using Type = std::string;
{variants}
}};",
        enum_format(&str_enum.name),
    );

    Ok(content)
}
