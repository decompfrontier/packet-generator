use std::{
    collections::HashMap,
    sync::{Arc, Weak},
};

#[derive(Clone, Debug)]
pub enum Encoding {
    String,
    Int,
}

#[derive(Clone, Debug)]
pub enum Definition {
    Json(Json),
    Struct(Json),
    IntEnum(IntEnum),
    StringEnum(StringEnum),
}

#[derive(Clone, Debug)]
pub enum JSONKey {
    String(String),
    UseUnderlying,
}

#[derive(Clone, Debug)]
pub struct Json {
    pub name: String,
    pub hash_name: Option<String>,
    pub fields: HashMap<Arc<str>, JsonField>,
}

#[derive(Clone, Debug)]
pub struct JsonField {
    pub name: Arc<str>,
    pub key: JSONKey,
    pub type_: DataType,
}

impl Json {
    pub fn new(name: String, hash_name: Option<String>) -> Self {
        Self {
            name,
            hash_name,
            fields: HashMap::new(),
        }
    }

    pub fn add_field(&mut self, field: JsonField) -> Option<JsonField> {
        self.fields.insert(field.name.clone(), field)
    }
}

#[derive(Clone, Debug)]
pub struct StringEnum {
    pub name: String,
    pub variants: HashMap<Arc<str>, StringEnumVariant>,
}

#[derive(Clone, Debug)]
pub struct StringEnumVariant {
    pub name: Arc<str>,
}

impl StringEnum {
    pub fn new(name: String) -> Self {
        Self {
            name,
            variants: HashMap::new(),
        }
    }

    pub fn add_variant(&mut self, field: StringEnumVariant) -> Option<StringEnumVariant> {
        self.variants.insert(field.name.clone(), field)
    }
}

#[derive(Clone, Debug)]
pub struct IntEnum {
    pub name: String,
    pub variants: HashMap<Arc<str>, IntEnumVariant>,
    pub start: i128,
}

#[derive(Clone, Debug)]
pub struct IntEnumVariant {
    pub name: Arc<str>,
}

impl IntEnum {
    pub fn new(name: String, start: i128) -> Self {
        Self {
            name,
            start,
            variants: HashMap::new(),
        }
    }

    pub fn add_variant(&mut self, field: IntEnumVariant) -> Option<IntEnumVariant> {
        self.variants.insert(field.name.clone(), field)
    }
}

#[derive(Clone, Debug)]
pub enum ArraySeparator {
    /// Array separated by ','
    ///
    /// ## Example
    ///
    /// [i32] = "1,3,4,5,6"
    Comma,

    /// Array separated by '@'
    ///
    /// ## Example
    ///
    /// [i32] = "1@3@4@5@6"
    At,

    /// Array separated by ':'
    ///
    /// ## Example
    ///
    /// [i32] = "1:3:4:5:6"
    Colon,
}

#[derive(Clone, Debug)]
pub enum DataType {
    I32 {
        encoding: Encoding,
    },

    U32 {
        encoding: Encoding,
    },

    I64 {
        encoding: Encoding,
    },

    U64 {
        encoding: Encoding,
    },

    F32 {
        encoding: Encoding,
    },

    F64,

    Bool {
        encoding: Encoding,
    },

    String,

    Datetime,

    DatetimeUnix,

    Map {
        key: Arc<DataType>,
        value: Arc<DataType>,
    },

    StringArray {
        inner_type: Arc<DataType>,
        separator: ArraySeparator,
    },

    Array {
        inner_type: Arc<DataType>,
    },

    SingleElementArray {
        inner_type: Arc<DataType>,
    },

    Definition(Weak<Definition>),

    Unknown(String),
}

#[derive(Debug, Clone, Default)]
pub struct DefinitionRegistry {
    pub definitions: HashMap<String, Arc<Definition>>,
    _private: std::marker::PhantomData<()>,
}

impl DefinitionRegistry {
    pub fn new() -> Self {
        Self {
            definitions: HashMap::new(),
            _private: std::marker::PhantomData {},
        }
    }

    pub fn insert(&mut self, definition: Definition) -> Option<Arc<Definition>> {
        match definition {
            Definition::Json(ref json) => self
                .definitions
                .insert(json.name.clone(), Arc::new(definition)),

            Definition::Struct(ref struct_) => self
                .definitions
                .insert(struct_.name.clone(), Arc::new(definition)),

            Definition::IntEnum(ref int_enum) => self
                .definitions
                .insert(int_enum.name.clone(), Arc::new(definition)),

            Definition::StringEnum(ref string_enum) => self
                .definitions
                .insert(string_enum.name.clone(), Arc::new(definition)),
        }
    }

    pub fn insert_and_get_weak(
        &mut self,
        definition: Definition,
    ) -> (Weak<Definition>, Option<Arc<Definition>>) {
        match definition {
            Definition::Json(ref json) => {
                let name = json.name.clone();
                let new_entry = Arc::new(definition);

                (
                    Arc::downgrade(&new_entry),
                    self.definitions.insert(name, new_entry),
                )
            }

            Definition::Struct(ref struct_) => {
                let name = struct_.name.clone();
                let new_entry = Arc::new(definition);

                (
                    Arc::downgrade(&new_entry),
                    self.definitions.insert(name, new_entry),
                )
            }

            Definition::IntEnum(ref int_enum) => {
                let name = int_enum.name.clone();
                let new_entry = Arc::new(definition);

                (
                    Arc::downgrade(&new_entry),
                    self.definitions.insert(name, new_entry),
                )
            }

            Definition::StringEnum(ref string_enum) => {
                let name = string_enum.name.clone();
                let new_entry = Arc::new(definition);

                (
                    Arc::downgrade(&new_entry),
                    self.definitions.insert(name, new_entry),
                )
            }
        }
    }

    pub fn find<S: AsRef<str>>(&self, name: S) -> Option<Arc<Definition>> {
        self.definitions.get(name.as_ref()).cloned()
    }

    pub fn find_weak<S: AsRef<str>>(&self, name: S) -> Option<Weak<Definition>> {
        self.definitions.get(name.as_ref()).map(Arc::downgrade)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn registry_can_handle_circular_definitions() {
        let mut definitions = DefinitionRegistry::new();

        {
            let field = JsonField {
                name: "bar".into(),
                key: JSONKey::String(String::from("bar")),
                type_: DataType::String,
            };

            let mut s = Json::new(String::from("Foo"), Some(String::from("avdsfdsf")));
            s.add_field(field);

            definitions.insert(Definition::Struct(s));
        };

        {
            let foo_struct = definitions
                .find_weak("Foo")
                .expect("Foo was inserted above.");

            let field = JsonField {
                name: "has_foo".into(),
                key: JSONKey::String(String::from("bar")),
                type_: DataType::Definition(foo_struct.clone()),
            };

            let mut s = Json::new(String::from("Bar"), Some(String::from("avfdsfdsf")));
            s.add_field(field);

            definitions.insert(Definition::Struct(s));
        };

        definitions.find("Bar").expect("Bar was inserted above.");
    }
}
