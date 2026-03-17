//! Utilities for converting a [`Document`](super::Document) into
//! the [IR](crate::intermediate) ([`DefinitionRegistry`]).

use std::sync::Arc;

use super::schema::EnumDefinition;
use crate::{
    intermediate::{
        DefinitionRegistry, Partial,
        schema::{
            self as intermediate, BoolEncoding as IntermediateBoolEncoding,
            DataType as IntermediateDataType, Definition, Encoding, Json, JsonField,
        },
    },
    kdl_parser::schema::{
        self, BoolEncoding, DataType as SchemaDataType, IntLikeEncoding,
        JsonDefinition as SchemaJsonDefinition, JsonField as SchemaJsonField,
    },
};

fn convert_datatype_recursive(
    type_: &schema::DataType,
    registry: &mut DefinitionRegistry<Partial>,
) -> intermediate::DataType {
    match type_ {
        schema::DataType::I32 { encoding } => match encoding {
            IntLikeEncoding::String => IntermediateDataType::I32 {
                encoding: Encoding::String,
            },
            IntLikeEncoding::Int => IntermediateDataType::I32 {
                encoding: Encoding::Int,
            },
        },

        SchemaDataType::U32 { encoding } => match encoding {
            IntLikeEncoding::String => IntermediateDataType::U32 {
                encoding: Encoding::String,
            },
            IntLikeEncoding::Int => IntermediateDataType::U32 {
                encoding: Encoding::Int,
            },
        },

        SchemaDataType::I64 { encoding } => match encoding {
            IntLikeEncoding::String => IntermediateDataType::I64 {
                encoding: Encoding::String,
            },
            IntLikeEncoding::Int => IntermediateDataType::I64 {
                encoding: Encoding::Int,
            },
        },

        SchemaDataType::U64 { encoding } => match encoding {
            IntLikeEncoding::String => IntermediateDataType::U64 {
                encoding: Encoding::String,
            },
            IntLikeEncoding::Int => IntermediateDataType::U64 {
                encoding: Encoding::Int,
            },
        },

        SchemaDataType::F32 { encoding } => match encoding {
            IntLikeEncoding::String => IntermediateDataType::F32 {
                encoding: Encoding::String,
            },
            IntLikeEncoding::Int => IntermediateDataType::F32 {
                encoding: Encoding::Int,
            },
        },

        SchemaDataType::F64 => IntermediateDataType::F64,

        SchemaDataType::Bool { encoding } => match encoding {
            BoolEncoding::String => IntermediateDataType::Bool {
                encoding: IntermediateBoolEncoding::String,
            },

            BoolEncoding::Int => IntermediateDataType::Bool {
                encoding: IntermediateBoolEncoding::Int,
            },

            BoolEncoding::Bool => IntermediateDataType::Bool {
                encoding: IntermediateBoolEncoding::Bool,
            },
        },

        SchemaDataType::Datetime => IntermediateDataType::Datetime,
        SchemaDataType::DatetimeUnix => IntermediateDataType::DatetimeUnix,

        SchemaDataType::String => IntermediateDataType::String,

        SchemaDataType::Array { inner, size } => IntermediateDataType::Array {
            inner_type: Arc::new(convert_datatype_recursive(inner, registry)),
            size: (*size).into(),
        },

        SchemaDataType::StringArray {
            inner,
            separator,
            size,
        } => {
            use crate::kdl_parser::schema;

            let size = (*size).into();

            match separator {
                schema::ArraySeparator::Comma => intermediate::DataType::StringArray {
                    separator: intermediate::ArraySeparator::Comma,
                    inner_type: Arc::new(convert_datatype_recursive(inner, registry)),
                    size,
                },

                schema::ArraySeparator::Pipe => intermediate::DataType::StringArray {
                    separator: intermediate::ArraySeparator::Pipe,
                    inner_type: Arc::new(convert_datatype_recursive(inner, registry)),
                    size,
                },

                schema::ArraySeparator::At => intermediate::DataType::StringArray {
                    separator: intermediate::ArraySeparator::At,
                    inner_type: Arc::new(convert_datatype_recursive(inner, registry)),
                    size,
                },

                schema::ArraySeparator::Colon => intermediate::DataType::StringArray {
                    separator: intermediate::ArraySeparator::Colon,
                    inner_type: Arc::new(convert_datatype_recursive(inner, registry)),
                    size,
                },
            }
        }

        SchemaDataType::Map { key, value } => intermediate::DataType::Map {
            key: Arc::new(convert_datatype_recursive(key, registry)),
            value: Arc::new(convert_datatype_recursive(value, registry)),
        },

        SchemaDataType::Custom { encoding, name } => {
            if let Some(idx) = registry.find_weak(name) {
                intermediate::DataType::Definition {
                    encoding: (*encoding).into(),
                    definition: idx,
                }
            } else {
                intermediate::DataType::Unknown {
                    encoding: (*encoding).into(),
                    name: name.to_owned(),
                }
            }
        }
    }
}

fn convert_json_datatype(
    schema_field: &SchemaJsonField,
    registry: &mut DefinitionRegistry<Partial>,
) -> IntermediateDataType {
    convert_datatype_recursive(&schema_field.r#type, registry)
}

pub fn add_enum_definitions(
    registry: &mut DefinitionRegistry<Partial>,
    enums: Vec<EnumDefinition>,
) {
    use crate::intermediate::schema::{IntEnum, StringEnum};

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
        }
    }
}

pub fn add_json_definitions(
    registry: &mut DefinitionRegistry<Partial>,
    structs: Vec<SchemaJsonDefinition>,
) {
    for struct_ in structs {
        let mut struct_def = Json::new(
            struct_.name,
            struct_.index,
            struct_.hash,
            struct_.doc,
            struct_.source_info,
            struct_.span,
        );

        for field in struct_.fields {
            struct_def.add_field(JsonField {
                index: field.index,
                name: field.name.clone().into(),
                key: field.key.clone(),
                type_: convert_json_datatype(&field, registry),
                optional: field.optional,
                doc: field.doc,
                span: field.span,
            });
        }

        registry.insert(Definition::Json(struct_def));
    }
}
