
from .generator import Generator
from .glazecpp import GlazeGenerator

class GeneratorFactory:
    """Factory pattern for generators."""
    @staticmethod
    def get(name: str) -> Generator:
        """Gets the generator based from the specified mapping."""
        GENERATOR_MAPPING :dict[str, type] = {
            "c++":          GlazeGenerator
        }        
        if not name in GENERATOR_MAPPING:
            raise Exception("Invalid generator: {}".format(name))
        
        return GENERATOR_MAPPING[name]()
