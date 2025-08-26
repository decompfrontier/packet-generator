# How to create a new packet or MST definition


## Introduction 
As Brave Frontier JSON data is shared between multiple projects, recreating the exact same input/output data every single time becomes a long task, with the cost of implicitly understanding the meaning of the code.

To address this issue, decompfrontier contains a simple python-based packet definition and generation to automatically generate packet and MST information.

In order to add a new definition, you'll need to clone the packet-generator repository (https://github.com/decompfrontier/packet-generator).

There are generally two types of information that you want to generate:
* Network definitions, these are the request/responses used by the game to talk with the server
* MST definitions, these are the definitions of the game data, such as units or items

You must have Python 3.10 or greater installed on your system.

## Adding a definition

First, you need to create a new python class that contains your new definition inside the specific field that you are adding.

For an MST, you can create a new python file in the `mst` directory, while for a Network request or response you can create a python file in the `net` directory, or alter any of the existing ones if you undercovered some information.

A general definition can be composed of the following data:
```
from schema import *

@keyjson(key_group = "6FrKacq7", single = True)
class SignalKey:
    key = { "Kn51uR4Y": str }
```


The JSON data can be configured in two ways:
* `keyjson`: It's a JSON that is encapsulated by a group key. This is the typical type you want when adding a new packet definition.
* `json`: It's a JSON without a group encapsulation. This is the typical type you want when adding a new MST.

The main difference between these two is the type of output that is generated, a keyjson (taking the example above) implies a JSON that is generated in this way:
```
"6FrKacq7": {
    "Kn51uR4Y": "SOMETHING"
}
```

While a JSON implies a simple JSON that is generated in this way:
```
{
    "Kn51uR4Y": "SOMETHING"
}
```

Notice the missing object identifier `6FrKacq7` which is what the group represents.

**Usage:**

```
@json(array: bool = True)

@keyjson(key_name: str, ...)
```

A keyjson inherits it's properties from a json.

| field name | description |
| ---------- | ------------ |
| key_name | This is the JSON group name (the encoded brave frontier name) |
| array | When this is set to true, it means that the json group is not an array of data but a single definition |

An array JSON means that the definition scheme can be used to contain more than one valorization, taking the case of the SignalKey definition before, this is what the output JSON would look like if the property array was set:
```
"6FrKacq7": {
    [
        {
            "Kn51uR4Y": "SOMETHING"
        }
    ]
}
```


While this is what the previous JSON would look like if the array property was set to False:
```
"6FrKacq7": {
    "Kn51uR4Y": "SOMETHING"
}
```

This is very usefull for definitions that encapsulate more fields, like unit or items are usually an array JSON.

## Writing a field

All fields for the parser are defined as a python dictionary

To write a field, simply do the following:

`field_name = { "brave frontier encrypted key name" : python datatype }`

Available data types to map to the JSON are:
| python name | native map behavour | JSON map behavour |
| ----------- | ------------------- | ----------------- |
| int | 32-bit unsigned int | number |
| str | string | string |
| intstr | 32-bit unsigned int converted to string | int | string |
| bool | boolean | boolean |
| long | 64-bit unsigned int | number |
| float | 32-bit floating point | float |
| floatstr | 32-bit floating point converted to string | float | string |
| strbool | boolean | string with value "1" or "0" |
| intbool | boolean | integer with value 1 or 0 |
| datetime | native datetime | string (data format: "2017-10-24 08:00:00") |
| datetimeunix | native unix datetime (time_t) | long (date format: unix timestamp) |
| list[type] | list of a type | array of a type (example: [ "a", "b" ]) |
| commalist[type] | list of strings | a string that contains a list of strings separated by a comma (example: "a,b,c" ), remember that the types should be string convertible |
| atlist[type] | list of types | a string that contains a list of a type separated by an at symbol (example: "a@b@c"), remember that the types should be string convertible |

### Custom attributes in fields
In certain situations, a field definition might not be enough, as the brave frontier
json accepts different configurations, in order to do so, you can just append a
custom key inside your field dictionary.

Custom keys:

| key name | Possible values | Types which it can applies | Default value if not specified | Description |
| -------- | --------------- | -------------------------- | ------------------------------ | ----------- |
| default | DefaultType.Omit | int, str | False | When the field does not have a value (the value considered is 0 on int while on string it's an empty one), if the value is Omit, the key/value is omitted entirely from the JSON generation. |
| quote | True/False | int, long, float | False | If it's true then the number will be represented as a quoted string |
| string | True/False | custom types | False | If it's true the custom type will be serialized as a string and not as a JSON object |

## Writing documentation

In order to also document what the fields actually do, the packet generator supports documentation in reStructuredText format.

For a class and enums, simply write your documentation as you would for any python class, for example:
```
@json(array = False)
class GmeAction:
    "Main packet of interaction between client and server." 
```

For class fields, create a new key called `doc` and place your documentation there, for example:
```
    body = { "" : GmeBody, "doc": "Body of the message." }
```

### Documenting enumerator fields

As enumerators doesn't have a key-value to append the documentation string to, the documentation is done by creating a new
dictionary called `__doc_fields__` which maps the *value* of the enumerator with it's documentation, for example:
```
class GmeErrorFlags(Flag):
    "Flags of the errors happend in the game."
    
    __doc_fields__ = {
        0: "No error oncurred.",
        1: "The server encountered an error.",
        4: "The custom command should be handled."
    }

    NoError = 0
    IsInError = 1
    ShouldHandleCommands = 4
```
