# How to create a new packet or MST definition


## Introduction
As Brave Frontier JSON data is shared between multiple projects, recreating the exact same input/output data every single time becomes a long task, with the cost of implicitly understanding the meaning of the code.

To address this issue, decompfrontier contains a simple [KDL-based](https://kdl.dev) packet schema
and a few generation utilities to automatically generate packet and MST information.

In order to add a new definition, you'll need to clone the packet-generator repository (https://github.com/decompfrontier/packet-generator).

There are generally two types of information that you want to generate:
* Network definitions, these are the request/responses used by the game to talk with the server
* MST definitions, these are the definitions of the game data, such as units or items

The definitions are currently rooted in `assets`, to add one either create a new KDL file or modify
an existing one.
For brevity, we assume that `net/foo.kdl` is in the directory `assets/net/foo.kdl`.
The main "entrypoint" is `net/handlers.kdl`.

You must have a Rust toolchain to compile the `packet-generator` library and CLI.

## Generating the output source files

To run the CLI from `cargo`, you can issue the following:

```sh
cargo run --release -- your arguments here
# Shorter: cargo r -r -- your arguments here
```
The binary `packet-generator` will be built inside `target/release`.
You can install it locally in your user's `PATH` by issuing `cargo install`:

```sh
cargo install --path packet-generator-cli

# You can now use:
packet-generator your-arguments-here
```

From now on we assume, for succintness in writing, that you call the `packet-generator`
binary directly, but everything can be done via the `cargo run` command without
locally installing the CLI.

Finally, to generate the source code for C++ using Glaze, issue the following:

```sh
# Full verbose command: packet-generator generate --cxx --glaze --input assets/net/handlers.kdl --output output/directory/for/the/definitions

# Shortened:
packet-generator generate --cxx --glaze -i assets/net/handlers.kdl -o output/directory/for/the/definitions
```

The `--help` for the `generate` sub-command will show the available options and
supported languages.

To add a definition: for an MST, you can create/modify a KDL file in the `assets/mst` directory,
while for a Network request or response you can create/modify a KDL file in the
`assets/net` directory.

## Basic syntax for definitions

First, please, read [kdl.dev](https://kdl.dev) to get familiar with the syntax
of the file format.
A general definition can be composed of the following data:

```kdl
// A comment.

json Foo {
    doc "Some documentation that will be included in the generated source"

    // A field/property named `bar` with type string.
    field bar type="str" {
        key "bar-key-in-json"

        doc "Some other documentation that will be included in the generated source"
    }

    field baz type="i32::int" {
        key "baz-key-in-json"

        doc "A 32-bit int represented as an integer."

    }
}
```

Note that the `doc` properties are _required_.
The above translates to the following JSON:

```json
{
    "bar-key-in-json": "a string value",
    "baz-key-in-json": 42
}
```

And, if using C++, to the following struct:

```cpp
/// Some documentation that will be included in the generated source
struct Foo {
    /// Some other that will be included in the generated source
    std::string bar;

    /// A 32-bit int represented as an integer.
    int32_t baz;
};
```

Moreover, one can define two types of enumerations: `int-enum` and `str-enum`.

```kdl
int-enum AnIntegerEnum start=42 {
    doc "This is a integer enum"

    variant first-variant {
        doc "The first integer variant of the enum"
    }

    variant second-variant {
        doc "The second integer variant of the enum, has value 12"
        value 12


        // The `value` property is optional, if unspecified the parser will
        // use incrementing numbers starting from the `start` property of the
        // node.
    }
}

str-enum AStringEnum {
    doc "This is a string enum"

    variant hello {
        doc "The first string variant of the enum"
        value "Hello"
    }

    variant world {
        doc "The first string variant of the enum"
        value "World!"
    }
}
```

These do not have a translation in JSON, but are used in the target language
to generate a type definition to prevent user errors.
For example, they translate to the following C++:

```cpp
/// This is a integer enum
enum class AnIntegerEnum {

    /// The first integer variant of the enum
    FirstVariant = 42,

    /// The second integer variant of the enum, has value 12
    SecondVariant = 12
};

/// This is a string enum
namespace AStringEnum {
    using Type = std::string;

    /// The first string variant of the enum
    constexpr const auto Hello = "Hello";

    /// The first string variant of the enum
    constexpr const auto World = "World!";
};
```

## Types in JSON fields

When defining a field for a JSON, one must specify a type in the `type` property
of the `field` node:

```kdl
json Foo {
    // ...

    field foo type="<HERE>" { /* ... */ }
}
```

The following are built-in **primitive types**:

| Syntax           | Description                     |
|------------------|---------------------------------|
| `i32`            | 32-bit integer                  |
| `u32`            | 32-bit unsigned integer         |
| `i64`            | 64-bit integer                  |
| `u64`            | 64-bit unsigned integer         |
| `f32`            | 32-bit float (`float`)          |
| `f64`            | 64-bit float (`double`)         |
| `bool`           | Boolean                         |
| `str`            | String of arbitrary length      |
| `datetime`       | A date and time                 |
| `datetime-unix`  | A date and time from Unix epoch |

Moreover, we support built-in **complex types** that can combine multiple types
together; most importantly _arrays_ and _associative maps_ (dictionaries).

| Syntax            | Description                     |
|-------------------|---------------------------------|
| `[Foo]`           | An array of `Foo`s              |
| `%{ Foo => Bar }` | A dictionary with `Foo` as keys and `Bar` as values |


For example, one could express an "array of dictionaries from strings to arrays of datetimes" as the following:

```
[%{ str => [datetime] }]
```


### User types

When specifying multiple definitions, one can use a previous custom definition
as a type:

```kdl
json Foo {
    // ...
}

json Bar {
    // ...

    field has-a-foo type="Foo" {
        // ...
    }
}
```

If the definition could not be found, `packet-generator` will issue an error.

### Type modifiers

Unfortunately, the previous types are not enough to represent all the ways
Brave Frontier serializes their JSONs.
For example, we need to express the difference between these two encodings
for an integer:

| As integer       | As string         |
|------------------|-------------------|
| `{ "foo": 42 }`  | `{ "foo": "42" }` |

In principle, we could (de)serialize the second case as a primitive `str`, but the
resulting code would be brittle and error-prone.
For this reason, one can attach one or more "modifier(s)" to a type.
At the end of a type declaration, specify a single modifier with
`::mod(param1, param2)` or many such with `::{mod1, mod2, mod3(param)}`.

Modifiers allow the parser to better understand the intent of the type by attaching
meta-information.
Unknown modifiers are currently silently dropped.
These are the currently supported modifiers:

 - When using `i32`, `u32`, `i64`, `u64` and `f32` (but not `f64`), one can specify
   either `int` or `str` to tell the parser if the type should be encoded
   as an integer or a string.
   Example:

   ```kdl
   field as-int type="i32::int" { /* ... */ }
   field as-str type="i32::str" { /* ... */ }

   // This is legal: the unknown modifiers (some, other, modifiers) will be ignored.
   field as-int-with-other-stuff type="i32::{int, some, other, modifiers}" { /* ... */ }
   ```

 - When using `bool`, if the modifiers `int` or `str` are present, the boolean
   will be encoded as either `0`/`1` or `"false"`/`"true"`.
   If none of those two modifiers are present (which is the **default**), then
   it will be encoded as a normal JSON boolean.
   Example:

   ```kdl
   field as-bool type="bool" { /* ... */ }
   field as-int type="bool::int" { /* ... */ }
   field as-str type="bool::str" { /* ... */ }

   // This is still a normal JSON bool.
   field still-as-bool type="bool::{these, are, discarded}" { /* ... */ }
   ```

 - When using an array, there are the following mutually-exclusive modifiers:
     - `sep(comma)`, `sep(at)`, `sep(colon)` will forcefully transform
       the content of the array in a string, where each value is separated by
       either a comma (`,`), an at (`@`) or a colon (`|`).
       A type such as `[i32::int]::sep(comma)` will parse the following JSON:
       ```json
       { "foo": "1,2,3,4,5,6" }
       ```
       The inner type *will* be converted to string:
       note that we specified the modifier `int` for `i32`, but it was
       superseded by the array.

     - `size(1)` forces the array to be of only one element.
     - (Optional) `size(n)` (with the **literal** `n`) is the default behavior.

   If none of the above modifiers are present, then the array will be serialized
   as a normal JSON array (equivalent to specifying `size(n)`).

Modifiers outside the ones above are **ignored**, in the future we may propagate
the information on the used modifiers to the generators so additional
meta-information could be used in the generation phase.

## Optional fields (poor man's sum-types)

The parser does not currently support [sum-types](https://en.wikipedia.org/wiki/Tagged_union)
(like Rust's `enum`) for JSONs.
One way to get around it is to use the `optional` child inside a field:

```kdl
json Foo {
    // ...
}

json Bar {
    // ...
}

json Action {
    field foo type=Foo {
        // ...
        optional #true
    }

    field bar type=Bar {
        // ...
        optional #true
    }
}
```

Note that, unfortunately, this accidentally expresses the non-existence of
_both_ fields, _not_ the _union_ between them.

<!---->
<!-- A keyjson inherits it's properties from a json. -->
<!---->
<!-- | field name | description | -->
<!-- | ---------- | ------------ | -->
<!-- | key_name | This is the JSON group name (the encoded brave frontier name) | -->
<!-- | array | When this is set to true, it means that the json group is not an array of data but a single definition | -->
<!---->
<!-- An array JSON means that the definition scheme can be used to contain more than one valorization, taking the case of the SignalKey definition before, this is what the output JSON would look like if the property array was set: -->
<!-- ``` -->
<!-- "6FrKacq7": { -->
<!--     [ -->
<!--         { -->
<!--             "Kn51uR4Y": "SOMETHING" -->
<!--         } -->
<!--     ] -->
<!-- } -->
<!-- ``` -->
<!---->
<!---->
<!-- While this is what the previous JSON would look like if the array property was set to False: -->
<!-- ``` -->
<!-- "6FrKacq7": { -->
<!--     "Kn51uR4Y": "SOMETHING" -->
<!-- } -->
<!-- ``` -->
<!---->
<!-- This is very useful for definitions that encapsulate more fields, like unit or items are usually an array JSON. -->
<!---->
<!-- ## Writing a field -->
<!---->
<!-- All fields for the parser are defined as a python dictionary -->
<!---->
<!-- To write a field, simply do the following: -->
<!---->
<!-- `field_name = { "brave frontier encrypted key name" : python datatype }` -->
<!---->
<!-- Available data types to map to the JSON are: -->
<!-- | python name | native map behaviour | JSON map behaviour | -->
<!-- | ----------- | ------------------- | ----------------- | -->
<!-- | int | 32-bit unsigned int | number | -->
<!-- | str | string | string | -->
<!-- | intstr | 32-bit unsigned int converted to string | int | string | -->
<!-- | bool | boolean | boolean | -->
<!-- | long | 64-bit unsigned int | number | -->
<!-- | float | 32-bit floating point | float | -->
<!-- | floatstr | 32-bit floating point converted to string | float | string | -->
<!-- | strbool | boolean | string with value "1" or "0" | -->
<!-- | intbool | boolean | integer with value 1 or 0 | -->
<!-- | datetime | native datetime | string (data format: "2017-10-24 08:00:00") | -->
<!-- | datetimeunix | native unix datetime (time_t) | long (date format: unix timestamp) | -->
<!-- | list[type] | list of a type | array of a type (example: [ "a", "b" ]) | -->
<!-- | commalist[type] | list of strings | a string that contains a list of strings separated by a comma (example: "a,b,c" ), remember that the types should be string convertible | -->
<!-- | atlist[type] | list of types | a string that contains a list of a type separated by an at symbol (example: "a@b@c"), remember that the types should be string convertible | -->
<!-- | colonlist[type] | list of types | a string that contains a list of a type separated by a colon (example: "a:b:c"), remember that the types should be string convertible | -->
<!---->
<!-- ### Custom attributes in fields -->
<!-- In certain situations, a field definition might not be enough, as the brave frontier -->
<!-- json accepts different configurations, in order to do so, you can just append a -->
<!-- custom key inside your field dictionary. -->
<!---->
<!-- Custom keys: -->
<!---->
<!-- | key name | Possible values | Types which it can applies | Default value if not specified | Description | -->
<!-- | -------- | --------------- | -------------------------- | ------------------------------ | ----------- | -->
<!-- | default | DefaultType.Omit | int, str | False | When the field does not have a value (the value considered is 0 on int while on string it's an empty one), if the value is Omit, the key/value is omitted entirely from the JSON generation. | -->
<!-- | quote | True/False | int, long, float | False | If it's true then the number will be represented as a quoted string | -->
<!-- | string | True/False | custom types | False | If it's true the custom type will be serialized as a string and not as a JSON object | -->
<!---->
<!-- ## Writing documentation -->
<!---->
<!-- In order to also document what the fields actually do, the packet generator supports documentation in reStructuredText format. -->
<!---->
<!-- For a class and enums, simply write your documentation as you would for any python class, for example: -->
<!-- ``` -->
<!-- @json(array = False) -->
<!-- class GmeAction: -->
<!--     "Main packet of interaction between client and server." -->
<!-- ``` -->
<!---->
<!-- For class fields, create a new key called `doc` and place your documentation there, for example: -->
<!-- ``` -->
<!--     body = { "" : GmeBody, "doc": "Body of the message." } -->
<!-- ``` -->
<!---->
<!-- ### Documenting enumerator fields -->
<!---->
<!-- As enumerators doesn't have a key-value to append the documentation string to, the documentation is done by creating a new -->
<!-- dictionary called `__doc_fields__` which maps the *value* of the enumerator with it's documentation, for example: -->
<!-- ``` -->
<!-- class GmeErrorFlags(Flag): -->
<!--     "Flags of the errors happened in the game." -->
<!---->
<!--     __doc_fields__ = { -->
<!--         0: "No error oncurred.", -->
<!--         1: "The server encountered an error.", -->
<!--         4: "The custom command should be handled." -->
<!--     } -->
<!---->
<!--     NoError = 0 -->
<!--     IsInError = 1 -->
<!--     ShouldHandleCommands = 4 -->
<!-- ``` -->
