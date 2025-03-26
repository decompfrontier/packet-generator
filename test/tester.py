from abc import ABC, abstractmethod
from schemawriter import SchemaWriter

class Tester(ABC):
    def output_file(self, filename: str, outputfile: str, language: str, schemas: list[type]):
        SchemaWriter.write(filename, outputfile, language, schemas)
        pass

    @abstractmethod
    def run(self):
        pass
