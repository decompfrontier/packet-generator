"""This module provides the schema structure"""
from dataclasses import dataclass, field
from enum import Enum, Flag
import inspect
from typing import TypeVar, MutableSequence

# define a generic typevar
_ST = TypeVar("_T")

# definitions of dummy types that are only used as distionctions for different generations

class strbool(type):
    """A boolean that is serialized as a string. ("0" or "1" defined as normal boolean for generators)"""
    pass

class intstr(type):
    """An integer that is serialized as a string. ("45" or "1" defined as a normal int for generators)"""
    pass

class floatstr(type):
    """A float that is serialized as a string. ("45.0" or "1.11" defined as a normal float for generators)"""
    pass

class long(type):
    """A 64-bit integer."""
    pass

class longstr(type):
    """A 64-bit integer that is serialized as a string. ("45" or "1" defined as a normal int for generators)"""
    pass

class double(type):
    """A 64-bit floating point."""
    pass

class intbool(type):
    """A boolean that is serialized as an integer. (0 or 1 defined as normal boolean for generators)"""
    pass

class datetimeunix(type):
    """A date time stored in unix epoch. (long with time like 1740763959)"""
    pass

class commalist(list[_ST]):
    """A list that is separated by commas 'a,b,c' """
    pass

class atlist(list[_ST]):
    """A list that is separated by at 'a@b@c' """
    pass

class ArrayStep(Enum):
    """This enum describes the types of Array that a JSON can contain."""

    NoArray = 0
    """The field has no array.
    
    Example in JSON:

        "key": "value",

    Example in generator (Python):

        key: str = "value"
    """

    Single = 1
    """The field is an array with ALWAYS one element.
    
    This can allow the generators to threat this array type
    as a single type to simplify programming the integrations.
    
    Example in JSON:

        [
            "key": "value"
        ]

    Example in generator (Python):

        key: str = "value"

    """

    Array = 2
    """This field is a normal array.
    
    Example in JSON:

        [
            "key": "value"
        ]
        
    Example in generator (Python):

        key: List[str] = [ "value" ]
    """

class DefaultType(Enum):
    """This enum describes the default action to do 
    when the field is empty.
    """

    Empty = 0
    """Simply
    report the default value.
    
    Example in JSON:
        "key": 0,
        "key": ""
    """

    Omit = 1
    """Omits this field when the field is empty.
    
    If the field is empty the parser will not produce the
    output."""

class ClassType(Enum):
    """This object represents the types of objects.
    
    For example:
        {
            "7zyHb5h9": 45,
            "B5JQyV8j": "test"
        }
    
    is a structure because multiple complex types are contained inside,
    while the enumerator is used only by the generators to guarantee more
    complex type/value checking.
    """

    Struct = 1
    """The JSON object is a structure"""

    Enumerator = 2
    """The type is an enumerator, this is a meta-type that does not exist in a JSON
     but can be used by generators to provide better value type checking.
    """

    EnumeratorString = 3
    """The type is an enumerator with a string, this is also a meta-type that does not exist
     int a JSON but can be used by generators to provide better value checking."""

@dataclass
class GeneratorField:
    """This object represents a field of a JSON.
    
    For example this represents a field like:
        "7zyHb5h9": 45,
        
    In such case the human name might be id and the key is 7zyHb5h9."""

    name: str = ""
    """Name of the field in human readable form."""

    key: str = ""
    """Name of the field (key) in Brave frontier form."""

    type_id: type = type(None)
    """Python native type of the field.
    
    For an enumerator this also includes the size like String or 8-byte integer.
    """

    default_action: DefaultType = DefaultType.Empty
    """Default action to do when the field is empty."""

    quoted: bool = False
    """Escapes the string (like MySQL string escaping sanitization)."""

    force_as_string: bool = False
    """Should the field be quoted as a String no matter it's type."""

@dataclass
class GeneratorStruct:
    """This object represents a structure to be converted to JSON.
    
    For example this represents an object like:

        "hV5vWu6C": {
            "7zyHb5h9": 45,
            "B5JQyV8j": "test"
        }
        
    Where the key is hV5vWu6C and name is NpcMst."""

    name: str = ""
    """Name of the object in human readable form."""

    key: str = ""
    """Name of the object (key) in Brave frontier form."""

    array_step: ArrayStep = ArrayStep.Array
    """Behaviour to apply for an array."""

    fields: list[GeneratorField] = field(default_factory=list)
    """Fields that are contained in this object."""

    class_type: ClassType = ClassType.Struct
    """Determines the type of the object."""


def processable(cls = None, /):
    """This decodator informs the generator that this file would be generated."""

    def wrap(cls):
        # __ir_declaration_line__ is an internal field to know if the type should be convertible to IR or not.
        # it contains the line number of the python declaration used for type declaration ordering (required for strong typed languages like C++).
        # Please note that the logic of pkprocess is very dummy and there are great possibilities that it might break on specific declaration conditions as python
        # doesn't have a real inspection, another alternative would be to parse the source code as string and get the line from there, but it sounds more complex than it should be
        cls.__ir_declaration_line__ = inspect.currentframe().f_back.f_lineno
        cls.__ir_array_step__ = ArrayStep.NoArray
        return cls

    if cls is None:
        return wrap

    cls2 = wrap(cls)
    cls2.__ir_declaration_line__ = inspect.currentframe().f_back.f_lineno
    return cls2

def json(cls = None, /, *, array = ArrayStep.NoArray):
    """This decorator rapresents a JSON fields or class.
    
    For example:
        {
            ...
        }

    :param cls: Base object to serialize
    :param array: Type of array step
    """

    def wrap(cls):
        cls.__ir_array_step__ = array
        cls.__ir_declaration_line__ = inspect.currentframe().f_back.f_lineno
        return cls
    
    if cls is None:
        return wrap
    
    cls2 = wrap(cls)
    cls2.__ir_declaration_line__ = inspect.currentframe().f_back.f_lineno
    return cls2

def keyjson(cls: object = None, /, *, key_group: str, array = ArrayStep.NoArray):
    """This decorator represents a JSON class that contains a custom key group.
    
    For example:
        {
            "key_group": [
                ...
            ]
        }

    :param cls: Base object to serialize
    :param key_group: Key group name (like 6FrKacq7)
    :param array: Type of array step
    """

    def _wrap(cls: object):
        cls.__ir_key_group__ = key_group
        q = json(cls, array = array)
        q.__ir_declaration_line__ = inspect.currentframe().f_back.f_lineno
        return q
    
    if cls is None:
        return _wrap

    cls2 = _wrap(cls)
    cls2.__ir_declaration_line__ = inspect.currentframe().f_back.f_lineno
    return cls2
