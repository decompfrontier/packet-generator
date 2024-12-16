from dataclasses import dataclass, field
from datetime import datetime
from enum import Enum
from abc import ABC, abstractmethod

class ArrayStep(Enum):
    NoArray = 0
    Single = 1
    Array = 2

class DefaultType(Enum):
    Nothing = 0
    Omit = 1
    Empty = 2

class ClassType(Enum):
    Struct = 1
    Enumerator = 2

@dataclass
class GeneratorField:
    name: str = ""
    key: str = ""
    type_id: type = type(None)
    default_action: DefaultType = DefaultType.Nothing
    quoted: bool = False
    force_as_string: bool = False

@dataclass
class GeneratorData:
    name: str = ""
    key: str = ""
    array_step: ArrayStep = ArrayStep.Array
    fields: list[GeneratorField] = field(default_factory=list)
    class_type: ClassType = ClassType.Struct

class Generator(ABC):
    @abstractmethod
    def get_start_mark(self, date: datetime, file_name: str) -> str:
        pass 

    @abstractmethod
    def get_end_mark(self) -> str:
        pass

    @abstractmethod
    def step(self, clz: GeneratorData) -> str:
        pass
