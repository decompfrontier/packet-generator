//! Intermediate

use std::{
    borrow::Borrow,
    collections::{BTreeMap, BTreeSet, btree_map::Values},
    sync::{Arc, Weak},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Encoding {
    String,
    Int,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum BoolEncoding {
    String,
    Int,
    Bool,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Definition {
    Json(Json),
    IntEnum(IntEnum),
    StringEnum(StringEnum),
}

impl Definition {
    #[must_use]
    pub const fn name(&self) -> &String {
        match self {
            Self::Json(json) => &json.name,
            Self::IntEnum(int_enum) => &int_enum.name,
            Self::StringEnum(string_enum) => &string_enum.name,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Json {
    pub index: usize,
    pub name: String,
    pub hash_name: Option<String>,
    pub fields: BTreeSet<JsonField>,
    pub doc: String,
}

impl Borrow<str> for Json {
    fn borrow(&self) -> &str {
        &self.name
    }
}

#[derive(Clone, Debug)]
pub struct JsonField {
    pub index: usize,
    pub name: Arc<str>,
    pub key: String,
    pub type_: DataType,
    pub optional: bool,
    pub doc: String,
}

impl Borrow<str> for JsonField {
    fn borrow(&self) -> &str {
        &self.name
    }
}

impl PartialEq for JsonField {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
            && self.name == other.name
            && self.key == other.key
            && self.optional == other.optional
            && self.doc == other.doc
    }
}

impl Eq for JsonField {}

impl PartialOrd for JsonField {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for JsonField {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.index.cmp(&other.index) {
            core::cmp::Ordering::Equal => {}
            ord => return ord,
        }
        match self.name.cmp(&other.name) {
            core::cmp::Ordering::Equal => {}
            ord => return ord,
        }
        match self.key.cmp(&other.key) {
            core::cmp::Ordering::Equal => {}
            ord => return ord,
        }
        self.optional.cmp(&other.optional)
    }
}

impl Json {
    #[must_use]
    pub const fn new(name: String, index: usize, hash_name: Option<String>, doc: String) -> Self {
        Self {
            index,
            name,
            hash_name,
            fields: BTreeSet::new(),
            doc,
        }
    }

    pub fn add_field(&mut self, field: JsonField) -> bool {
        self.fields.insert(field)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct StringEnum {
    pub index: usize,
    pub name: String,
    pub doc: String,
    pub variants: BTreeSet<StringEnumVariant>,
}

#[derive(Clone, Debug, Eq, PartialOrd, Ord)]
pub struct StringEnumVariant {
    pub index: usize,
    pub name: Arc<str>,
    pub doc: String,
    pub value: String,
}

impl Borrow<str> for StringEnumVariant {
    fn borrow(&self) -> &str {
        &self.name
    }
}

impl PartialEq for StringEnumVariant {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

impl StringEnum {
    #[must_use]
    pub const fn new(name: String, index: usize, doc: String) -> Self {
        Self {
            index,
            name,
            doc,
            variants: BTreeSet::new(),
        }
    }

    pub fn add_variant(&mut self, field: StringEnumVariant) -> bool {
        self.variants.insert(field)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct IntEnum {
    pub index: usize,
    pub name: String,
    pub start: i128,
    pub doc: String,
    pub variants: BTreeSet<IntEnumVariant>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct IntEnumVariant {
    pub index: usize,
    pub name: Arc<str>,
    pub doc: String,
    pub value: Option<i128>,
}

impl Borrow<str> for IntEnumVariant {
    fn borrow(&self) -> &str {
        &self.name
    }
}

impl IntEnum {
    #[must_use]
    pub const fn new(name: String, index: usize, doc: String, start: i128) -> Self {
        Self {
            index,
            name,
            start,
            doc,
            variants: BTreeSet::new(),
        }
    }

    pub fn add_variant(&mut self, field: IntEnumVariant) -> bool {
        self.variants.insert(field)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
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
        encoding: BoolEncoding,
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
    definitions: BTreeMap<String, Arc<Definition>>,

    _private: std::marker::PhantomData<()>,
}

impl DefinitionRegistry {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            definitions: BTreeMap::new(),
            _private: std::marker::PhantomData {},
        }
    }

    pub fn insert(&mut self, definition: Definition) -> Option<Arc<Definition>> {
        match definition {
            Definition::Json(ref json) => self
                .definitions
                .insert(json.name.clone(), Arc::new(definition)),

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

    pub fn all_definitions(&self) -> Values<'_, String, Arc<Definition>> {
        self.definitions.values()
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
                index: 0,
                name: "bar".into(),
                key: String::from("bar"),
                type_: DataType::String,
                optional: false,
                doc: String::from("some documentation"),
            };

            let mut s = Json::new(
                String::from("Foo"),
                0,
                Some(String::from("avdsfdsf")),
                String::from("some documentation"),
            );
            s.add_field(field);

            definitions.insert(Definition::Json(s));
        };

        {
            let foo_struct = definitions
                .find_weak("Foo")
                .expect("Foo was inserted above.");

            let field = JsonField {
                index: 0,
                name: "has_foo".into(),
                key: String::from("bar"),
                type_: DataType::Definition(foo_struct),
                optional: false,
                doc: String::from("some documentation"),
            };

            let mut s = Json::new(
                String::from("Bar"),
                1,
                Some(String::from("avfdsfdsf")),
                String::from("some documentation"),
            );
            s.add_field(field);

            definitions.insert(Definition::Json(s));
        };

        definitions.find("Bar").expect("Bar was inserted above.");
    }
}
