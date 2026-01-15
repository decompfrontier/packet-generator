from gen.generator import Generator
from schema import *
from datetime import datetime

class CppGlazeType(Generator):
    """
    Generates Glaze JSON C++ mapping classes.
    """

    def get_start_mark(self, file_name: str) -> str:
        return "" # Mapping generators doesn't have start marks, they are inherited from CppSingleType

    def add_import(self, imp: str, output_dir: str) -> str:
        return "" # Mapping generators doesn't have imports, this is inherited from CppSingleType

    def get_end_mark(self) -> str:
        return "" # Mapping generators doesn't have imports, this is inherited from CppSingleType

    def get_extension(self) -> str:
        return "" # Mapping generators doesn't have imports, this is inherited from CppSingleType

    def step(self, struct: GeneratorStruct) -> str:
        if struct.class_type == ClassType.Struct:
            # generate a struct
            return self._step_struct(struct)
        elif struct.class_type == ClassType.Enumerator:
            return self._step_enum(struct)
        else:
            return self._step_enum_string(struct)
