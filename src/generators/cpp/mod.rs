use std::path::PathBuf;

mod glaze;

pub use glaze::GlazeGenerator;

use atomicow::CowArc;
use itertools::Itertools;
use stringcase::Caser;

use crate::generators::{Addon, GeneratedSource, GenerationError, Generator, WithAddons};

use crate::intermediate::{DefinitionRegistry, schema::*};

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

fn struct_field_format(string: &str) -> String {
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
                Definition::Json(json) => Ok(format!("struct {};", struct_format(&json.name))),
                Definition::IntEnum(int_enum) => {
                    Ok(format!("enum class {};", enum_format(&int_enum.name)))
                }
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
        CowArc::Owned(struct_field_format(&definition.name).into())
    }

    fn int_enum_name<'a>(&'a self, definition: &'a IntEnum) -> CowArc<'a, str> {
        CowArc::Owned(enum_format(&definition.name).into())
    }

    fn int_enum_variant_name<'a>(&'a self, definition: &'a IntEnumVariant) -> CowArc<'a, str> {
        CowArc::Owned(enum_variant_format(&definition.name).into())
    }

    fn string_enum_name<'a>(&'a self, definition: &'a StringEnum) -> CowArc<'a, str> {
        CowArc::Owned(enum_format(&definition.name).into())
    }

    fn string_enum_variant_name<'a>(
        &'a self,
        definition: &'a StringEnumVariant,
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

        // Custom `float`/`double` to support C++20 floating point.
        DataType::F32 { .. } => Ok(String::from("pkg::float32")),
        DataType::F64 => Ok(String::from("pkg::float64")),

        DataType::Bool { .. } => Ok(String::from("bool")),

        DataType::String => Ok(String::from("std::string")),

        DataType::Datetime | DataType::DatetimeUnix => Ok(String::from("pkg::chrono_time")),

        DataType::Map { key, value } => {
            let key = convert_datatype(key, registry)?;
            let value = convert_datatype(value, registry)?;

            Ok(format!("std::unordered_map<{key}, {value}>"))
        }

        DataType::Array {
            inner_type,
            size: ArraySize::Dynamic,
        } => {
            let inner = convert_datatype(inner_type, registry)?;

            Ok(format!("std::vector<{inner}>"))
        }

        DataType::Array {
            inner_type,
            size: ArraySize::Fixed(size),
        } => {
            let inner = convert_datatype(inner_type, registry)?;

            match size.get() {
                1 => Ok(inner),
                n => Ok(format!("std::array<{inner}, {n}>")),
            }
        }

        DataType::StringArray {
            inner_type,
            separator: _,
            size: ArraySize::Dynamic,
        } => {
            let inner = convert_datatype(inner_type, registry)?;

            Ok(format!("pkg::string_list<{inner}>"))
        }

        DataType::StringArray {
            inner_type: _,
            separator: _,
            size: ArraySize::Fixed(_),
        } => {
            // TODO(Arves): Implement this.
            todo!("Fixed array size for string arrays are not implemented yet.");
        }

        DataType::Definition {
            definition: weak,
            encoding: JsonEncoding::Json,
        } => {
            let definition = registry.get(*weak);
            match *definition {
                Definition::StringEnum(ref str_enum) => {
                    Ok(format!("{}::Type", enum_format(&str_enum.name)))
                }

                Definition::Json(ref json) => Ok(struct_format(&json.name)),
                Definition::IntEnum(ref int_enum) => Ok(enum_format(&int_enum.name)),
            }
        }

        DataType::Definition {
            definition: _,
            encoding: JsonEncoding::String,
        } => {
            // TODO(anri): Handle JSON string encoding
            todo!("String-encoded JSONs are not implemented yet.");
        }

        DataType::Unknown { name: other, .. } => match registry.find(other) {
            Some((definition, _idx)) => match definition {
                Definition::StringEnum(str_enum) => {
                    Ok(format!("{}::Type", enum_format(&str_enum.name)))
                }

                Definition::Json(json) => Ok(struct_format(&json.name)),
                Definition::IntEnum(int_enum) => Ok(enum_format(&int_enum.name)),
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
            let mut datatype = convert_datatype(&field.type_, registry)?;

            if field.optional {
                datatype = format!("std::optional<{datatype}>");
            }

            let name = struct_field_format(&field.name);
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
            let doc = split_documentation(&variant.doc, 1);

            format!("\n{doc}\n{TAB}{name} = {start},")
        })
        .unwrap_or_default();

    let variants_str = variants_iter
        .map(|variant| {
            let name = enum_variant_format(&variant.name);
            let maybe_val = variant.value.map(|v| format!(" = {v}")).unwrap_or_default();
            let doc = split_documentation(&variant.doc, 1);

            format!("\n{doc}\n{TAB}{name}{maybe_val}")
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
            let doc = split_documentation(&variant.doc, 1);
            let val = &variant.value;
            format!("\n{doc}\n{TAB}constexpr const auto {name} = \"{val}\";")
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kdl_parser::SourceInfo;

    use std::sync::{Arc, LazyLock};

    struct SyntheticData {
        json: Json,
        int_enum: IntEnum,
        str_enum: StringEnum,
    }

    fn build_synthetic_definitions() -> SyntheticData {
        let source = Arc::new(SourceInfo::new(String::new(), String::new()));

        let arced_name: Arc<str> = TEST_NAME.into();

        let mut json = Json::new(
            TEST_NAME.to_owned(),
            0,
            String::new(),
            source,
            (0, 0).into(),
        );

        json.add_field(JsonField {
            index: 0,
            name: arced_name.clone(),
            key: String::new(),
            type_: DataType::String,
            optional: false,
            doc: String::new(),
            span: (0, 0).into(),
        });

        let mut int_enum = IntEnum::new(TEST_NAME.to_owned(), 0, String::new(), 0);

        int_enum.add_variant(IntEnumVariant {
            index: 0,
            name: arced_name.clone(),
            doc: String::new(),
            value: None,
        });

        let mut str_enum = StringEnum::new(TEST_NAME.to_owned(), 0, String::new());

        str_enum.add_variant(StringEnumVariant {
            index: 0,
            name: arced_name,
            doc: String::new(),
            value: String::new(),
        });

        SyntheticData {
            json,
            int_enum,
            str_enum,
        }
    }

    const TEST_NAME: &str = "any-random_namedSequence-Of_characters";
    const TEST_GENERATOR: CxxGenerator = CxxGenerator::new();
    static TEST_DATA: LazyLock<SyntheticData> = LazyLock::new(build_synthetic_definitions);

    #[test]
    fn can_format_structs() {
        let expected = "AnyRandomNamedSequenceOfCharacters";
        assert_eq!(struct_format(TEST_NAME), expected);

        assert_eq!(TEST_GENERATOR.json_name(&TEST_DATA.json).as_ref(), expected);
    }

    #[test]
    fn can_format_struct_fields() {
        let expected = "any_random_named_sequence_of_characters";
        assert_eq!(struct_field_format(TEST_NAME), expected);

        assert_eq!(
            TEST_GENERATOR
                .json_field_name(TEST_DATA.json.fields.get(TEST_NAME).unwrap())
                .as_ref(),
            expected
        );
    }

    #[test]
    fn can_format_enums() {
        let expected = "AnyRandomNamedSequenceOfCharacters";
        assert_eq!(enum_format(TEST_NAME), expected);

        assert_eq!(
            TEST_GENERATOR.int_enum_name(&TEST_DATA.int_enum).as_ref(),
            expected
        );

        assert_eq!(
            TEST_GENERATOR
                .string_enum_name(&TEST_DATA.str_enum)
                .as_ref(),
            expected
        );
    }

    #[test]
    fn can_format_enum_variants() {
        let expected = "AnyRandomNamedSequenceOfCharacters";
        assert_eq!(enum_variant_format(TEST_NAME), expected);

        assert_eq!(
            TEST_GENERATOR
                .int_enum_variant_name(TEST_DATA.int_enum.variants.get(TEST_NAME).unwrap())
                .as_ref(),
            expected
        );

        assert_eq!(
            TEST_GENERATOR
                .string_enum_variant_name(TEST_DATA.str_enum.variants.get(TEST_NAME).unwrap())
                .as_ref(),
            expected
        );
    }

    #[test]
    fn can_add_addons() {
        #[derive(Debug, Clone)]
        struct MockAddon {}
        impl Addon for MockAddon {
            type For = CxxGenerator;
        }

        let mut generator = CxxGenerator::new();
        assert!(generator.addons.is_empty());

        generator.add_addon(MockAddon {});
        generator.add_addon(MockAddon {});
        generator.add_addon(MockAddon {});

        assert_eq!(generator.addons.len(), 3);
    }

    #[test]
    fn can_split_documentation() {
        const DOC: &str = r#"# Foo

Bar.
    Baz."#;

        assert_eq!(
            split_documentation(DOC, 0),
            r#"/// # Foo
///
/// Bar.
///     Baz."#
        );

        assert_eq!(
            split_documentation(DOC, 2),
            r#"        /// # Foo
        ///
        /// Bar.
        ///     Baz."#
        );
    }
}
