pub mod schema;
mod schema_validator;
mod type_parser;

pub struct Document(RawDocument);

pub use schema_validator::{ValidationError, validate};

use crate::{intermediate::DefinitionRegistry, kdl_parser::schema::RawDocument};

pub fn raw_parse_kdl<S: AsRef<str>>(
    document: S,
) -> Result<schema::RawDocument, facet_kdl::KdlDeserializeError> {
    facet_kdl::from_str(document.as_ref())
}

mod document_to_intermediate {
    use std::sync::Arc;

    use super::schema::EnumDefinition;
    use crate::{
        intermediate::{
            DataType as IntermediateDataType, Definition, DefinitionRegistry, Encoding, Struct,
            StructField,
        },
        kdl_parser::schema::{
            DataDefinition, DataProperty, DataType as SchemaDataType, TypeEncoding,
        },
    };

    fn convert_datatype(
        schema_field: &DataProperty,
        registry: &mut DefinitionRegistry,
    ) -> IntermediateDataType {
        use crate::intermediate;
        use crate::kdl_parser::schema;

        fn recursive_matching(
            type_: &schema::DataType,
            encoding: Option<schema::TypeEncoding>,
            registry: &mut DefinitionRegistry,
        ) -> intermediate::DataType {
            match type_ {
                schema::DataType::I32 => {
                    match encoding.expect("todo: error handling for encoding") {
                        TypeEncoding::String => IntermediateDataType::I32 {
                            encoding: Encoding::String,
                        },
                        TypeEncoding::Int => IntermediateDataType::I32 {
                            encoding: Encoding::Int,
                        },
                    }
                }

                SchemaDataType::U32 => match encoding.expect("todo: error handling for encoding") {
                    TypeEncoding::String => IntermediateDataType::U32 {
                        encoding: Encoding::String,
                    },
                    TypeEncoding::Int => IntermediateDataType::U32 {
                        encoding: Encoding::Int,
                    },
                },

                SchemaDataType::I64 => match encoding.expect("todo: error handling for encoding") {
                    TypeEncoding::String => IntermediateDataType::I64 {
                        encoding: Encoding::String,
                    },
                    TypeEncoding::Int => IntermediateDataType::I64 {
                        encoding: Encoding::Int,
                    },
                },

                SchemaDataType::U64 => match encoding.expect("todo: error handling for encoding") {
                    TypeEncoding::String => IntermediateDataType::U64 {
                        encoding: Encoding::String,
                    },
                    TypeEncoding::Int => IntermediateDataType::U64 {
                        encoding: Encoding::Int,
                    },
                },

                SchemaDataType::F32 => match encoding.expect("todo: error handling for encoding") {
                    TypeEncoding::String => IntermediateDataType::F32 {
                        encoding: Encoding::String,
                    },
                    TypeEncoding::Int => IntermediateDataType::F32 {
                        encoding: Encoding::Int,
                    },
                },

                SchemaDataType::F64 => match encoding.expect("todo: error handling for encoding") {
                    TypeEncoding::String => IntermediateDataType::F64 {
                        encoding: Encoding::String,
                    },
                    TypeEncoding::Int => IntermediateDataType::F64 {
                        encoding: Encoding::Int,
                    },
                },

                SchemaDataType::Bool => {
                    match encoding.expect("todo: error handling for encoding") {
                        TypeEncoding::String => IntermediateDataType::Bool {
                            encoding: Encoding::String,
                        },
                        TypeEncoding::Int => IntermediateDataType::Bool {
                            encoding: Encoding::Int,
                        },
                    }
                }

                SchemaDataType::Datetime => IntermediateDataType::Datetime,
                SchemaDataType::String => IntermediateDataType::String,
                SchemaDataType::Json => todo!(),

                SchemaDataType::Array { inner, separator } => {
                    use crate::kdl_parser::schema;

                    // NOTE(anri):
                    // Patch the encoding for array types to always be String,
                    // since the game doesn't support anything else anyway.
                    let encoding = Some(schema::TypeEncoding::String);

                    match separator {
                        schema::ArraySeparator::Comma => intermediate::DataType::Array {
                            separator: intermediate::ArraySeparator::Comma,
                            inner_type: Arc::new(recursive_matching(inner, encoding, registry)),
                        },

                        schema::ArraySeparator::At => intermediate::DataType::Array {
                            separator: intermediate::ArraySeparator::At,
                            inner_type: Arc::new(recursive_matching(inner, encoding, registry)),
                        },

                        schema::ArraySeparator::Colon => intermediate::DataType::Array {
                            separator: intermediate::ArraySeparator::Colon,
                            inner_type: Arc::new(recursive_matching(inner, encoding, registry)),
                        },
                    }
                }

                SchemaDataType::JsonArray { type_hint: _ } => todo!(),

                SchemaDataType::SingleElementArray(data_type) => {
                    intermediate::DataType::SingleElementArray {
                        inner_type: Arc::new(recursive_matching(data_type, encoding, registry)),
                    }
                }

                SchemaDataType::Map { key, value } => intermediate::DataType::Map {
                    key: Arc::new(recursive_matching(key, encoding, registry)),
                    value: Arc::new(recursive_matching(value, encoding, registry)),
                },

                SchemaDataType::Tuple(_data_types) => todo!(),

                SchemaDataType::Custom(s) => {
                    if let Some(def) = registry.find_weak(s) {
                        intermediate::DataType::Definition(def)
                    } else {
                        intermediate::DataType::Unknown(s.to_owned())
                    }
                }
            }
        }

        recursive_matching(&schema_field.r#type, schema_field.encoding, registry)
    }

    pub fn add_enum_definitions(registry: &mut DefinitionRegistry, enums: Vec<EnumDefinition>) {
        use crate::intermediate::{IntEnum, StringEnum};

        for enum_ in enums {
            match enum_ {
                EnumDefinition::StringEnum(enum_def) => {
                    let def = Definition::StringEnum(StringEnum::from(enum_def));
                    registry.insert(def);
                }

                EnumDefinition::IntEnum(enum_def) => {
                    let def = Definition::IntEnum(IntEnum::from(enum_def));
                    registry.insert(def);
                }
            };
        }
    }

    pub fn add_struct_definitions(registry: &mut DefinitionRegistry, structs: Vec<DataDefinition>) {
        for struct_ in structs {
            let mut struct_def = Struct::new(struct_.name, struct_.hash);

            for field in struct_.fields {
                struct_def.add_field(StructField {
                    name: field.name.clone().into(),
                    hash_name: field.hash.clone(),
                    type_: convert_datatype(&field, registry),
                });
            }

            registry.insert(Definition::Struct(struct_def));
        }
    }
}

pub fn document_to_definitions(document: Document) -> DefinitionRegistry {
    let mut registry = DefinitionRegistry::new();

    document_to_intermediate::add_enum_definitions(&mut registry, document.0.enums);
    document_to_intermediate::add_struct_definitions(&mut registry, document.0.data);

    registry
}
