from dataclasses import dataclass, field
from datetime import datetime
from enum import Enum
from abc import ABC, abstractmethod

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
     but can be used on generators to provide better value type checking.
    """

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

    doc: str = ""
    """Explains what this field does."""

@dataclass
class GeneratorData:
    """This object represents an object to be converted to JSON.
    
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

    doc: str = ""
    """Exaplains what this object does."""

class Generator(ABC):
    """This interface exposes the generic methods for implementing a source generator for BF data."""
    @abstractmethod
    def get_start_mark(self, date: datetime, file_name: str) -> str:
        """
        Gets the mark to apply at the beginning of the file.
        This can be a simple comment or something specific of the generator.

        :param date: Date when the file is generated
        :param file_name: Original file name of the name
        :return: Start mark
        """
        pass

    @abstractmethod
    def get_end_mark(self) -> str:
        """
        Gets the mark to apply at the end of the file.
        This can be a simple comment or something specific of the generator.

        :return: Start mark
        """
        pass

    @abstractmethod
    def step(self, clz: GeneratorData) -> str:
        """
        Performs one step of the generator and generates the string rapresentation of the
        specified generator class.

        :param clz: Class to generate
        :return: String conversion of the class
        """
        pass
