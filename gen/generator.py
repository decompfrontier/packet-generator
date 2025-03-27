from datetime import datetime
from abc import ABC, abstractmethod
from schema import GeneratorData

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

    @abstractmethod
    def get_extension(self) -> str:
        """
        Gets the preferred extension of this generator.
        """
        pass
