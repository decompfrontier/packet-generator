
from gen.data import Generator
from .glazecpp import GlazeGenerator

class GeneratorFactory:
    @staticmethod
    def get(name: str) -> Generator:
        match name:
            case "c++":
                return GlazeGenerator()
            case _:
                raise Exception("Invalid generator: {}".format(name))
