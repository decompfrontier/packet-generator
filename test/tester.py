from abc import ABC, abstractmethod
from schemawriter import SchemaWriter
from gen import GeneratorFactory

class Tester(ABC):
    def output_file(self, filename: str, outputfile: str, language: str, schemas: list[type]):
        g = GeneratorFactory.get(language)
        SchemaWriter.write(filename, outputfile, schemas, g)

    @abstractmethod
    def run(self):
        pass
